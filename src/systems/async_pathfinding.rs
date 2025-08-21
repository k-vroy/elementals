use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use crate::systems::world_gen::TerrainMap;
use crate::systems::pawn::PawnTarget;
use crate::systems::pathfinding_cache::PathfindingCache;

/// Component that holds a running pathfinding task
#[derive(Component)]
pub struct PathfindingTask {
    pub task: Task<PathfindingResult>,
    pub start: (f32, f32),
    pub goal: (f32, f32),
    pub size: f32,
    pub request_id: u64,
}

/// Result of a pathfinding computation
#[derive(Clone)]
pub struct PathfindingResult {
    pub path: Option<Vec<(f32, f32)>>,
    pub start: (f32, f32),
    pub goal: (f32, f32),
    pub size: f32,
    pub request_id: u64,
}

/// Component to mark entities that need pathfinding
#[derive(Component)]
pub struct PathfindingRequest {
    pub start: (f32, f32),
    pub goal: (f32, f32),
    pub size: f32,
    pub priority: PathfindingPriority,
    pub request_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathfindingPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Resource to track pathfinding request IDs
#[derive(Resource, Default)]
pub struct PathfindingRequestCounter {
    pub next_id: u64,
}

impl PathfindingRequestCounter {
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}

/// Resource for global pathfinding cache - shared across async tasks
#[derive(Resource)]
pub struct GlobalPathfindingCache {
    cache: PathfindingCache,
}

impl Default for GlobalPathfindingCache {
    fn default() -> Self {
        Self {
            cache: PathfindingCache::new(),
        }
    }
}

impl PathfindingRequest {
    pub fn new(start: (f32, f32), goal: (f32, f32), size: f32) -> Self {
        Self {
            start,
            goal,
            size,
            priority: PathfindingPriority::Normal,
            request_id: 0, // Will be set by the system
        }
    }

    pub fn with_priority(mut self, priority: PathfindingPriority) -> Self {
        self.priority = priority;
        self
    }
}

/// System to spawn pathfinding tasks for entities with PathfindingRequest
pub fn spawn_pathfinding_tasks(
    mut commands: Commands,
    terrain_map: Res<TerrainMap>,
    mut request_counter: ResMut<PathfindingRequestCounter>,
    request_query: Query<(Entity, &PathfindingRequest), Without<PathfindingTask>>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    
    // Sort by priority (high priority first)
    let mut requests: Vec<_> = request_query.iter().collect();
    requests.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    for (entity, request) in requests {
        // Generate unique request ID
        let request_id = request_counter.next_id();
        
        // Clone data for the async task
        let terrain_clone = terrain_map.clone();
        let start = request.start;
        let goal = request.goal;
        let size = request.size;
        
        // Spawn async pathfinding task
        let task = task_pool.spawn(async move {
            // Perform pathfinding computation in background thread
            let path = terrain_clone.find_path_for_size(start, goal, size);
            
            PathfindingResult {
                path,
                start,
                goal,
                size,
                request_id,
            }
        });
        
        // Replace PathfindingRequest with PathfindingTask
        commands.entity(entity)
            .remove::<PathfindingRequest>()
            .insert(PathfindingTask {
                task,
                start,
                goal,
                size,
                request_id,
            });
    }
}

/// System to handle completed pathfinding tasks
pub fn handle_completed_pathfinding(
    mut commands: Commands,
    mut completed_query: Query<(Entity, &mut PathfindingTask)>,
) {
    for (entity, mut pathfinding_task) in completed_query.iter_mut() {
        // Check if task is finished
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(&mut pathfinding_task.task)) {
            // Task completed, process result
            if let Some(path) = result.path {
                // Create PawnTarget with the computed path
                let target_pos = Vec3::new(result.goal.0, result.goal.1, 100.0);
                let mut pawn_target = PawnTarget::new(target_pos);
                pawn_target.set_path(path);
                
                commands.entity(entity)
                    .remove::<PathfindingTask>()
                    .insert(pawn_target);
            } else {
                // No path found, just remove the task
                commands.entity(entity).remove::<PathfindingTask>();
            }
        }
    }
}

/// System to spawn cached pathfinding tasks (uses global cache)
pub fn spawn_cached_pathfinding_tasks(
    mut commands: Commands,
    terrain_map: Res<TerrainMap>,
    mut global_cache: ResMut<GlobalPathfindingCache>,
    mut request_counter: ResMut<PathfindingRequestCounter>,
    request_query: Query<(Entity, &PathfindingRequest), Without<PathfindingTask>>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    
    // Sort by priority (high priority first)
    let mut requests: Vec<_> = request_query.iter().collect();
    requests.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    for (entity, request) in requests {
        // Generate unique request ID
        let request_id = request_counter.next_id();
        
        // Check cache first (synchronously, should be fast)
        let start_tile = terrain_map.world_to_tile_coords(request.start.0, request.start.1);
        let goal_tile = terrain_map.world_to_tile_coords(request.goal.0, request.goal.1);
        
        if let (Some(start_tile), Some(goal_tile)) = (start_tile, goal_tile) {
            if let Some(cached_path) = global_cache.cache.get_path(start_tile, goal_tile, request.size) {
                // Cache hit! Use cached result immediately
                if let Some(path) = cached_path {
                    let target_pos = Vec3::new(request.goal.0, request.goal.1, 100.0);
                    let mut pawn_target = PawnTarget::new(target_pos);
                    pawn_target.set_path(path.clone());
                    
                    commands.entity(entity)
                        .remove::<PathfindingRequest>()
                        .insert(pawn_target);
                } else {
                    // Cached "no path" result
                    commands.entity(entity).remove::<PathfindingRequest>();
                }
                continue;
            }
        }
        
        // Cache miss, spawn async task
        let terrain_clone = terrain_map.clone();
        let start = request.start;
        let goal = request.goal;
        let size = request.size;
        
        let task = task_pool.spawn(async move {
            // Perform pathfinding computation in background thread
            let path = terrain_clone.find_path_for_size(start, goal, size);
            
            PathfindingResult {
                path,
                start,
                goal,
                size,
                request_id,
            }
        });
        
        commands.entity(entity)
            .remove::<PathfindingRequest>()
            .insert(PathfindingTask {
                task,
                start,
                goal,
                size,
                request_id,
            });
    }
}

/// System to handle completed cached pathfinding tasks and update cache
pub fn handle_completed_cached_pathfinding(
    mut commands: Commands,
    terrain_map: Res<TerrainMap>,
    mut global_cache: ResMut<GlobalPathfindingCache>,
    mut completed_query: Query<(Entity, &mut PathfindingTask)>,
) {
    for (entity, mut pathfinding_task) in completed_query.iter_mut() {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(&mut pathfinding_task.task)) {
            // Update cache with result
            if let (Some(start_tile), Some(goal_tile)) = (
                terrain_map.world_to_tile_coords(result.start.0, result.start.1),
                terrain_map.world_to_tile_coords(result.goal.0, result.goal.1)
            ) {
                global_cache.cache.cache_path(start_tile, goal_tile, result.size, result.path.clone(), &terrain_map);
            }
            
            // Process result
            if let Some(path) = result.path {
                let target_pos = Vec3::new(result.goal.0, result.goal.1, 100.0);
                let mut pawn_target = PawnTarget::new(target_pos);
                pawn_target.set_path(path);
                
                commands.entity(entity)
                    .remove::<PathfindingTask>()
                    .insert(pawn_target);
            } else {
                commands.entity(entity).remove::<PathfindingTask>();
            }
        }
    }
}

/// Cleanup system to remove stale pathfinding requests/tasks
pub fn cleanup_stale_pathfinding(
    mut commands: Commands,
    request_query: Query<Entity, (With<PathfindingRequest>, Without<Transform>)>,
    task_query: Query<Entity, (With<PathfindingTask>, Without<Transform>)>,
) {
    // Remove pathfinding requests/tasks from entities that no longer exist or don't have Transform
    for entity in request_query.iter().chain(task_query.iter()) {
        commands.entity(entity).despawn();
    }
}

/// Helper function to request pathfinding for an entity
pub fn request_pathfinding(
    commands: &mut Commands,
    entity: Entity,
    start: (f32, f32),
    goal: (f32, f32),
    size: f32,
) {
    commands.entity(entity).insert(PathfindingRequest::new(start, goal, size));
}

/// Helper function to request high-priority pathfinding (e.g., player input)
pub fn request_priority_pathfinding(
    commands: &mut Commands,
    entity: Entity,
    start: (f32, f32),
    goal: (f32, f32),
    size: f32,
    priority: PathfindingPriority,
) {
    commands.entity(entity).insert(
        PathfindingRequest::new(start, goal, size).with_priority(priority)
    );
}