use std::{time::Duration, f32::consts::PI};

use crate::{cursor::{Position, Velocity}, osc_encode_decode::ObjectParams};

#[derive(Debug, Clone)]
pub struct Object {
    session_id: i32,
    class_id: i32,
    time: Duration,
    position: Position,
    angle: f32,
    velocity: Velocity,
    rotation_speed: f32,
    acceleration: f32,
    rotation_acceleration: f32
}

impl Object {
    /// Creates a new [Object]
    /// # Arguments
    /// * `time` - the time of creation
    /// * `session_id` - a unique session ID
    /// * `class_id` - the object's class ID
    /// * `position` - a normalized [Position]
    /// * `angle` - an angle in radians
    pub fn new(time: Duration, session_id: i32, class_id: i32, position: Position, angle: f32) -> Self {
        Self {
            session_id,
            class_id,
            time,
            position,
            velocity: Velocity::default(),
            acceleration: 0f32,
            angle,
            rotation_speed: 0f32,
            rotation_acceleration: 0f32
        }
    }

    /// Returns this [Object] with motion
    /// # Arguments
    /// * `velocity` - a normalized [Velocity]
    /// * `rotation_speed` - a rotation speed in turns per second
    /// * `acceleration` - a normalized acceleration
    /// * `rotation_acceleration` - a roation acceleration in radians turn per second squared
    pub fn with_motion(
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

    pub fn get_position(&self) -> &Position {
        &self.position
    }

    pub fn get_x_position(&self) -> f32 {
        self.position.x
    }

    pub fn get_y_position(&self) -> f32 {
        self.position.y
    }

    pub fn get_velocity(&self) -> &Velocity {
        &self.velocity
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

    pub fn update(&mut self, time: Duration, position: Position, angle: f32) {
        let delta_time = (time - self.time).as_secs_f32();

        let distance = position.distance_from(&self.position);
        let delta_x = position.x - self.position.x;
        let delta_y = position.y - self.position.y;

        let last_speed = self.velocity.get_speed();
        let speed = distance / delta_time;

        self.velocity = Velocity {
            x: delta_x / delta_time,
            y: delta_y / delta_time,
        };
        
        self.acceleration = (speed - last_speed) / delta_time;
        self.position = position;

        
        let delta_turn = (angle - self.angle) / (2. * PI);
        let rotation_speed = delta_turn / delta_time;

        self.rotation_acceleration = (rotation_speed - self.rotation_speed) / delta_time;
        self.rotation_speed = rotation_speed;

        self.time = time;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_values(
        &mut self,
        time: Duration,
        class_id: i32,
        position: Position,
        angle: f32,
        velocity: Velocity,
        rotation_speed: f32,
        acceleration: f32,
        rotation_acceleration: f32,
    ) {
        self.time = time;
        self.class_id = class_id;
        self.position = position;
        self.angle = angle;
        self.velocity = velocity;
        self.rotation_speed = rotation_speed;
        self.acceleration = acceleration;
        self.rotation_acceleration = rotation_acceleration;
    }

    pub fn update_from_params(&mut self, time: Duration, params: ObjectParams) {
        self.time = time;
        self.class_id = params.class_id;
        self.position = Position{x: params.x_pos, y: params.y_pos};
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
            position: Position{x: params.x_pos, y: params.y_pos},
            angle: params.angle,
            velocity: Velocity{x: params.x_vel, y: params.y_vel},
            rotation_speed: params.rotation_speed,
            acceleration: params.acceleration,
            rotation_acceleration: params.rotation_acceleration
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

    use crate::{cursor::Position, object::Object};

    #[test]
    fn object_update() {
        let mut object = Object::new(Duration::default(), 0, 0, Position { x: 0., y: 0. }, 0.);

        object.update(
            Duration::from_secs(1),
            Position { x: 1., y: 1. },
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
