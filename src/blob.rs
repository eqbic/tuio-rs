use std::time::Duration;

use crate::{cursor::{Point, State, Velocity}, osc_encode_decode::BlobParams};

#[derive(Debug)]
pub struct Blob {
    session_id: i32,
    time: Duration,
    path: Vec<Point>,
    velocity: Velocity,
    acceleration: f32,
    angle: f32,
    angular_speed: f32,
    angular_acceleration: f32,
    width: f32,
    height: f32,
    area: f32,
    state: State,
}

impl Blob {
    pub fn new(
        time: Duration,
        session_id: i32,
        point: Point,
        angle: f32,
        width: f32,
        height: f32,
        area: f32,
    ) -> Self {
        Self {
            session_id,
            time,
            path: Vec::from([point]),
            velocity: Velocity::default(),
            acceleration: 0f32,
            angle,
            angular_speed: 0f32,
            angular_acceleration: 0f32,
            width,
            height,
            area,
            state: State::Added,
        }
    }

    pub fn with_movement(
        mut self,
        velocity: Velocity,
        angular_speed: f32,
        acceleration: f32,
        angular_acceleration: f32,
    ) -> Self {
        self.velocity = velocity;
        self.angular_speed = angular_speed;
        self.acceleration = acceleration;
        self.angular_acceleration = angular_acceleration;
        self
    }

    pub fn get_time(&self) -> Duration {
        self.time
    }

    pub fn update(
        &mut self,
        time: Duration,
        point: Point,
        angle: f32,
        width: f32,
        height: f32,
        area: f32,
    ) {
        todo!()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_values(
        &mut self,
        time: Duration,
        position: Point,
        angle: f32,
        width: f32,
        height: f32,
        area: f32,
        velocity: Velocity,
        angular_speed: f32,
        acceleration: f32,
        angular_acceleration: f32,
    ) {
        self.time = time;
        self.path.push(position);
        self.angle = angle;
        self.width = width;
        self.height = height;
        self.area = area;
        self.velocity = velocity;
        self.angular_speed = angular_speed;
        self.acceleration = acceleration;
        self.angular_acceleration = angular_acceleration;
    }

    pub fn update_from_params(&mut self, time: Duration, params: BlobParams) {
        self.time = time;
        self.path.push(Point{x: params.x_pos, y: params.y_pos});
        self.angle = params.angle;
        self.width = params.width;
        self.height = params.height;
        self.area = params.area;
        self.velocity = Velocity{x: params.x_vel, y: params.y_vel};
        self.angular_speed = params.angular_speed;
        self.acceleration = params.acceleration;
        self.angular_acceleration = params.angular_acceleration;
    }

    pub fn get_session_id(&self) -> i32 {
        self.session_id
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

    pub fn get_state(&self) -> State {
        self.state
    }
}

impl PartialEq for Blob {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.get_x_position() == other.get_x_position()
            && self.get_x_position() == other.get_y_position()
            && self.angle == other.angle
            && self.velocity == other.velocity
            && self.angular_speed == other.angular_speed
            && self.acceleration == other.acceleration
            && self.angular_acceleration == other.angular_acceleration
            && self.width == other.width
            && self.height == other.height
            && self.area == other.area
    }
}

impl From<(Duration, BlobParams)> for Blob {
    fn from((time, params): (Duration, BlobParams)) -> Self {
        Self {
            session_id: params.session_id,
            path: vec![Point{x: params.x_pos, y: params.y_pos}],
            angle: params.angle,
            width: params.width,
            height: params.height,
            area: params.area,
            velocity: Velocity{x: params.x_vel, y: params.y_vel},
            angular_speed: params.angular_speed,
            acceleration: params.acceleration,
            angular_acceleration: params.angular_acceleration,
            time,
            state: State::Added,
        }
    }
}

impl From<BlobParams> for Blob {
    fn from(params: BlobParams) -> Self {
        (Duration::default(), params).into()
    }
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, f32::consts::SQRT_2};

    use crate::{blob::Blob, cursor::Point};

    #[test]
    fn blob_update() {
        let mut blob = Blob::new(
            Duration::default(),
            0,
            Point { x: 0., y: 0. },
            0.,
            0.,
            0.,
            0.,
        );

        blob.update(
            Duration::from_secs(1),
            Point { x: 1., y: 1. },
            90f32.to_radians(),
            0.5,
            0.5,
            0.25,
        );

        assert_eq!(blob.get_x_position(), 1.);
        assert_eq!(blob.get_y_position(), 1.);
        assert_eq!(blob.get_x_velocity(), 1.);
        assert_eq!(blob.get_y_velocity(), 1.);
        assert_eq!(blob.get_acceleration(), SQRT_2);
        assert_eq!(blob.get_angular_speed(), 90f32.to_radians());
        assert_eq!(blob.get_acceleration(), 90f32.to_radians());
        assert_eq!(blob.get_width(), 0.5);
        assert_eq!(blob.get_height(), 0.5);
        assert_eq!(blob.get_area(), 0.25);
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
