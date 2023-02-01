use std::time::Duration;

use crate::cursor::{Point, State, Velocity};

#[derive(Debug)]
pub struct Object {
    session_id: i32,
    class_id: i32,
    time: Duration,
    path: Vec<Point>,
    angle: f32,
    velocity: Velocity,
    angular_speed: f32,
    acceleration: f32,
    angular_acceleration: f32,
    state: State,
}

impl Object {
    pub fn new(time: Duration, session_id: i32, class_id: i32, point: Point, angle: f32) -> Self {
        Self {
            session_id,
            class_id,
            time,
            path: Vec::from([point]),
            velocity: Velocity::default(),
            acceleration: 0f32,
            angle,
            angular_speed: 0f32,
            angular_acceleration: 0f32,
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

    pub fn get_session_id(&self) -> i32 {
        self.session_id
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

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn update(&mut self, time: Duration, position: Point, angle: f32) {
        todo!()
    }

    pub fn update_values(
        &mut self,
        time: Duration,
        class_id: i32,
        position: Point,
        angle: f32,
        velocity: Velocity,
        angular_speed: f32,
        acceleration: f32,
        angular_acceleration: f32,
    ) {
        self.time = time;
        self.class_id = class_id;
        self.path.push(position);
        self.angle = angle;
        self.velocity = velocity;
        self.angular_speed = angular_speed;
        self.acceleration = acceleration;
        self.angular_acceleration = angular_acceleration;
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.class_id == other.class_id
            && self.get_x_position() == other.get_x_position()
            && self.get_x_position() == other.get_y_position()
            && self.angle == other.angle
            && self.velocity == other.velocity
            && self.angular_speed == other.angular_speed
            && self.acceleration == other.acceleration
            && self.angular_acceleration == other.angular_acceleration
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{cursor::Point, object::Object};

    #[test]
    fn object_update() {
        let mut object = Object::new(Duration::default(), 0, 0, Point { x: 0., y: 0. }, 0.);

        object.update(
            Duration::from_secs(1),
            Point { x: 1., y: 1. },
            90f32.to_radians(),
        );

        assert_eq!(object.get_x_position(), 1.);
        assert_eq!(object.get_y_position(), 1.);
        assert_eq!(object.get_x_velocity(), 1.);
        assert_eq!(object.get_y_velocity(), 1.);
        assert_eq!(object.get_acceleration(), 1.4142135);
        assert_eq!(object.get_angular_speed(), 90f32.to_radians());
        assert_eq!(object.get_acceleration(), 90f32.to_radians());
    }
}
