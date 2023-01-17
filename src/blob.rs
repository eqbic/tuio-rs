use std::time::{Instant, Duration};

use crate::cursor::{Point, Velocity, State};

pub struct Blob {
    session_id: i32,
    instant: Instant,
    path: Vec<Point>,
    velocity: Velocity,
    acceleration: f32,
    angle: f32,
    angular_speed: f32,
    angular_acceleration: f32,
    width: f32,
    height: f32,
    area: f32,
    duration: Duration,
    state: State
}

impl Blob {
    #[allow(clippy::too_many_arguments)]
    pub fn new(instant: Instant, session_id: i32, point: Point, angle: f32, width: f32, height: f32, area: f32) -> Self {
        Self { 
            session_id,
            instant,
            path: Vec::from([point]),
            velocity: Velocity::default(),
            acceleration: 0f32,
            angle,
            angular_speed: 0f32,
            angular_acceleration: 0f32,
            width,
            height,
            area,
            duration: instant.elapsed(),
            state: State::Added
        }
    }

    pub fn update(&mut self, point: Point, angle: f32, width: f32, height: f32, area: f32) {
        todo!()
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

    pub fn get_width(&self) -> f32 {
        self.width
    }
    
    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn get_pixel_width(&self, screen_width: u16) -> u16 {
        (self.width * screen_width as f32) as u16
    }
    
    pub fn get_pixel_height(&self, screen_height: u16) -> u16 {
        (self.width * screen_height as f32) as u16
    }

    pub fn get_area(&self) -> f32 {
        self.area
    }
}

/*

impl Moving for Blob {
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

impl Rotating for Blob {
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

impl Update<(Point, f32, f32, f32, f32)> for Blob {
    fn update(&mut self, (point, angle, width, height, area): (Point, f32, f32, f32, f32)) {
        self.width = width;
        self.height = height;
        self.area = area;
        self.update((point, angle));
    }
}

impl Update<(Point, f32, f32, f32, f32, Velocity, f32, f32, f32)> for Blob {
    fn update(&mut self, (point, angle, width, height, area, velocity, angular_speed, acceleration, angular_acceleration): (Point, f32, f32, f32, f32, Velocity, f32, f32, f32)) {
        self.width = width;
        self.height = height;
        self.area = area;
        self.update((point, angle, velocity, angular_speed, acceleration, angular_acceleration));
    }
}
*/