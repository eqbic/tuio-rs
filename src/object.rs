use std::{time::Duration, f32::consts::PI};

use crate::{cursor::{Point, State, Velocity}, osc_encode_decode::ObjectParams};

#[derive(Debug)]
pub struct Object {
    session_id: i32,
    class_id: i32,
    time: Duration,
    path: Vec<Point>,
    angle: f32,
    velocity: Velocity,
    rotation_speed: f32,
    acceleration: f32,
    rotation_acceleration: f32,
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
            rotation_speed: 0f32,
            rotation_acceleration: 0f32,
            state: State::Added,
        }
    }

    pub fn with_movement(
        mut self,
        velocity: Velocity,
        rotation_speed: f32,
        acceleration: f32,
        rotation_acceleration: f32,
    ) -> Self {
        self.velocity = velocity;
        self.rotation_speed = rotation_speed;
        self.acceleration = acceleration;
        self.rotation_acceleration = rotation_acceleration;
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

    /// Returns the angle in radians
    pub fn get_angle(&self) -> f32 {
        self.angle
    }

    /// Returns the rotation speed in turn per seconds
    pub fn get_rotation_speed(&self) -> f32 {
        self.rotation_speed
    }

    /// Returns the rotation acceleration in turn per seconds squared
    pub fn get_rotation_acceleration(&self) -> f32 {
        self.rotation_acceleration
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn update(&mut self, time: Duration, position: Point, angle: f32) {
        let delta_time = (time - self.time).as_secs_f32();
        let last_position = self.path.last().unwrap();

        let distance = position.distance_from(last_position);
        let delta_x = position.x - last_position.x;
        let delta_y = position.y - last_position.y;

        let last_speed = self.velocity.get_speed();
        let speed = distance / delta_time;

        self.velocity = Velocity {
            x: delta_x / delta_time,
            y: delta_y / delta_time,
        };
        
        self.acceleration = (speed - last_speed) / delta_time;
        self.path.push(position);

        
        let delta_turn = (angle - self.angle) / (2. * PI);
        let rotation_speed = delta_turn / delta_time;

        self.rotation_acceleration = (rotation_speed - self.rotation_speed) / delta_time;
        self.rotation_speed = rotation_speed;

        self.time = time;

        self.state = if self.acceleration > 0f32 {
            State::Accelerating
        } else if self.acceleration < 0f32 {
            State::Decelerating
        } else {
            State::Stopped
        };
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_values(
        &mut self,
        time: Duration,
        class_id: i32,
        position: Point,
        angle: f32,
        velocity: Velocity,
        rotation_speed: f32,
        acceleration: f32,
        rotation_acceleration: f32,
    ) {
        self.time = time;
        self.class_id = class_id;
        self.path.push(position);
        self.angle = angle;
        self.velocity = velocity;
        self.rotation_speed = rotation_speed;
        self.acceleration = acceleration;
        self.rotation_acceleration = rotation_acceleration;
    }

    pub fn update_from_params(&mut self, time: Duration, params: ObjectParams) {
        self.time = time;
        self.class_id = params.class_id;
        self.path.push(Point{x: params.x_pos, y: params.y_pos});
        self.angle = params.angle;
        self.velocity = Velocity{x: params.x_vel, y: params.y_vel};
        self.rotation_speed = params.rotation_speed;
        self.acceleration = params.acceleration;
        self.rotation_acceleration = params.rotation_acceleration;
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
            && self.rotation_speed == other.rotation_speed
            && self.acceleration == other.acceleration
            && self.rotation_acceleration == other.rotation_acceleration
    }
}

impl From<(Duration, ObjectParams)> for Object {
    fn from((time, params): (Duration, ObjectParams)) -> Self {
        Self {
            session_id: params.session_id,
            class_id: params.class_id,
            time,
            path: vec![Point{x: params.x_pos, y: params.y_pos}],
            angle: params.angle,
            velocity: Velocity{x: params.x_vel, y: params.y_vel},
            rotation_speed: params.rotation_speed,
            acceleration: params.acceleration,
            rotation_acceleration: params.rotation_acceleration,
            state: State::Added,
        }
    }
}

impl From<ObjectParams> for Object {
    fn from(params: ObjectParams) -> Self {
        (Duration::default(), params).into()
    }
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, f32::consts::SQRT_2};

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
        assert_eq!(object.get_acceleration(), SQRT_2);
        assert_eq!(object.get_rotation_speed(), 0.25);
        assert_eq!(object.get_rotation_acceleration(), 0.25);
    }
}
