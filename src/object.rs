use std::time::{Instant, Duration};

use crate::cursor::{Point, Velocity, State};

pub struct Object {
    session_id: i32,
    class_id: i32,
    instant: Instant,
    path: Vec<Point>,
    velocity: Velocity,
    acceleration: f32,
    angle: f32,
    angular_speed: f32,
    angular_acceleration: f32,
    duration: Duration,
    state: State
}

impl Object {
    pub fn new(instant: Instant, session_id: i32, class_id: i32, point: Point, angle: f32) -> Self {
        Self { 
            session_id,
            class_id,
            instant,
            path: Vec::from([point]),
            velocity: Velocity::default(),
            acceleration: 0f32,
            angle,
            angular_speed: 0f32,
            angular_acceleration: 0f32,
            duration: instant.elapsed(),
            state: State::Added
        }
    }

    pub fn get_class_id(&self) -> i32 {
        self.class_id
    }

    pub fn get_x_position(&self) -> f32 {
        self.path.last().unwrap().x
    }

    pub fn get_y_position(&self) -> f32 {
        self.path.last().unwrap().y
    }

    pub fn get_x_velocity(&self) -> f32 {
        self.velocity.x
    }

    pub fn get_y_velocity(&self) -> f32 {
        self.velocity.y
    }

    pub fn get_acceleration(&self) -> f32 {
        self.acceleration
    }

    pub fn get_angle(&self) -> f32 {
        self.angle
    }

    pub fn get_angular_speed(&self) -> f32 {
        self.angular_speed
    }

    pub fn get_angular_acceleration(&self) -> f32 {
        self.angular_acceleration
    }

    pub fn update(&mut self, position: Point, angle: f32) {
        todo!()
    }
}

/*
// #[derive(Clone, Copy)]
pub struct Object {
    id: u32,
    class_id: u32,
    instant: Instant,
    source: Source,
    path: Vec<Point>,
    velocity: Velocity,
    acceleration: f32,
    angle: f32,
    angular_speed: f32,
    angular_acceleration: f32,
    duration: Duration,
    state: State
}

impl Object {
    pub fn new(instant: Instant, id: u32, class_id: u32, source: Source, point: Point, angle: f32) -> Self {
        Self { 
            id,
            class_id,
            instant,
            source,
            path: Vec::from([point]),
            velocity: Velocity::default(),
            acceleration: 0f32,
            angle,
            angular_speed: 0f32,
            angular_acceleration: 0f32,
            duration: instant.elapsed(),
            state: State::Added
        }
    }
}

impl Moving for Object {
    fn get_delta_time(&self) -> f32 {
        (self.instant.elapsed() - self.duration).as_secs_f32()
    }

    fn get_velocity(&self) -> Velocity {
        self.velocity
    }

    fn get_last_point(&self) -> &Point {
        self.path.last().unwrap()
    }

    fn set_point(&mut self, point: Point) {
        self.path.push(point);
    }

    fn get_path(&self) -> &[Point] {
        &self.path
    }

    fn get_speed(&self) -> f32 {
        self.velocity.get_speed()
    }

    fn get_acceleration(&self) -> f32 {
        self.acceleration
    }

    fn get_state(&self) -> State {
        self.state
    }

    fn is_moving(&self) -> bool {
        self.state == State::Accelerating || self.state == State::Decelerating || self.state == State::Rotating
    }

    fn set_velocity(&mut self, velocity: Velocity) {
        self.velocity = velocity;
    }

    fn set_acceleration(&mut self, acceleration: f32) {
        self.acceleration = acceleration;
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }

    fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    fn get_instant(&self) -> Instant {
        self.instant
    }
}

impl Rotating for Object {
    fn get_angle(&self) -> f32 {
        self.angle
    }

    fn set_angle(&mut self, angle: f32) {
        self.angle = angle;
    }

    fn get_angular_speed(&self) -> f32 {
        self.angular_speed
    }

    fn set_angular_speed(&mut self, angular_speed: f32) {
        self.angular_speed = angular_speed;
    }
    
    fn set_angular_acceleration(&mut self, angular_acceleration: f32) {
        self.angular_acceleration = angular_acceleration;
    }
}
*/