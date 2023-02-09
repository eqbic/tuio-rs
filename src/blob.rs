use std::{time::Duration, f32::consts::PI};

use crate::{
    cursor::{Position, Velocity},
    osc_encode_decode::BlobParams,
};

#[derive(Debug, Clone)]
pub struct Blob {
    session_id: i32,
    time: Duration,
    position: Position,
    velocity: Velocity,
    acceleration: f32,
    angle: f32,
    rotation_speed: f32,
    rotation_acceleration: f32,
    width: f32,
    height: f32,
    area: f32
}

impl Blob {
    /// Creates a new [Blob]
    /// # Arguments
    /// * `time` - the time of creation
    /// * `session_id` - a unique session ID
    /// * `position` - a normalized [Position]
    /// * `angle` - an angle in radians
    /// * `width` - a normalized width
    /// * `height` - a normalized height
    /// * `area` - a normalized area
    pub fn new(
        time: Duration,
        session_id: i32,
        position: Position,
        angle: f32,
        width: f32,
        height: f32,
        area: f32,
    ) -> Self {
        Self {
            session_id,
            time,
            position,
            velocity: Velocity::default(),
            acceleration: 0f32,
            angle,
            rotation_speed: 0f32,
            rotation_acceleration: 0f32,
            width,
            height,
            area
        }
    }

    /// Returns this [Blob] with motion
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

    pub fn update(
        &mut self,
        time: Duration,
        position: Position,
        angle: f32,
        width: f32,
        height: f32,
        area: f32,
    ) {
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

        self.width = width;
        self.height = height;
        self.area = area;

        self.time = time;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_values(
        &mut self,
        time: Duration,
        position: Position,
        angle: f32,
        width: f32,
        height: f32,
        area: f32,
        velocity: Velocity,
        rotation_speed: f32,
        acceleration: f32,
        rotation_acceleration: f32,
    ) {
        self.time = time;
        self.position = position;
        self.angle = angle;
        self.width = width;
        self.height = height;
        self.area = area;
        self.velocity = velocity;
        self.rotation_speed = rotation_speed;
        self.acceleration = acceleration;
        self.rotation_acceleration = rotation_acceleration;
    }

    pub fn update_from_params(&mut self, time: Duration, params: BlobParams) {
        self.update_values(
            time,
            Position {
                x: params.x_pos,
                y: params.y_pos,
            },
            params.angle,
            params.width,
            params.height,
            params.area,
            Velocity {
                x: params.x_vel,
                y: params.y_vel,
            },
            params.rotation_speed,
            params.acceleration,
            params.rotation_acceleration,
        );
    }

    pub fn get_session_id(&self) -> i32 {
        self.session_id
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

    /// Returns the normalized width
    pub fn get_width(&self) -> f32 {
        self.width
    }

    /// Returns the normalized height
    pub fn get_height(&self) -> f32 {
        self.height
    }

    /// Returns the width in screen space
    pub fn get_pixel_width(&self, screen_width: u16) -> u16 {
        (self.width * screen_width as f32) as u16
    }

    /// Returns the height in screen space
    pub fn get_pixel_height(&self, screen_height: u16) -> u16 {
        (self.width * screen_height as f32) as u16
    }

    /// Returns the normalized area
    pub fn get_area(&self) -> f32 {
        self.area
    }
}

impl PartialEq for Blob {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.get_x_position() == other.get_x_position()
            && self.get_x_position() == other.get_y_position()
            && self.angle == other.angle
            && self.velocity == other.velocity
            && self.rotation_speed == other.rotation_speed
            && self.acceleration == other.acceleration
            && self.rotation_acceleration == other.rotation_acceleration
            && self.width == other.width
            && self.height == other.height
            && self.area == other.area
    }
}

impl From<(Duration, BlobParams)> for Blob {
    fn from((time, params): (Duration, BlobParams)) -> Self {
        Self {
            session_id: params.session_id,
            position: Position {
                x: params.x_pos,
                y: params.y_pos,
            },
            angle: params.angle,
            width: params.width,
            height: params.height,
            area: params.area,
            velocity: Velocity {
                x: params.x_vel,
                y: params.y_vel,
            },
            rotation_speed: params.rotation_speed,
            acceleration: params.acceleration,
            rotation_acceleration: params.rotation_acceleration,
            time
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
    use std::{f32::consts::SQRT_2, time::Duration};

    use crate::{blob::Blob, cursor::Position};

    #[test]
    fn blob_update() {
        let mut blob = Blob::new(
            Duration::default(),
            0,
            Position { x: 0., y: 0. },
            0.,
            0.,
            0.,
            0.,
        );

        blob.update(
            Duration::from_secs(1),
            Position { x: 1., y: 1. },
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
        assert_eq!(blob.get_rotation_speed(), 0.25);
        assert_eq!(blob.get_rotation_acceleration(), 0.25);
        assert_eq!(blob.get_width(), 0.5);
        assert_eq!(blob.get_height(), 0.5);
        assert_eq!(blob.get_area(), 0.25);
    }
}