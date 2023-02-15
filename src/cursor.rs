use std::time::Duration;

#[derive(Default, Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn distance_from(&self, position: &Position) -> f32 {
        let dx = self.x - position.x;
        let dy = self.y - position.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn get_speed(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cursor {
    pub(crate) session_id: i32,
    pub(crate) position: Position,
    pub(crate) velocity: Velocity,
    pub(crate) acceleration: f32,
}

impl Cursor {
    /// Creates a new [Cursor]
    /// # Arguments
    /// * `session_id` - a unique session ID
    /// * `position` - a normalized [Position]
    pub fn new(session_id: i32, position: Position) -> Self {
        Self {
            session_id,
            position,
            velocity: Velocity::default(),
            acceleration: 0f32,
        }
    }

    /// Returns this [Cursor] with motion
    /// # Arguments
    /// * `velocity` - a normalized [Velocity]
    /// * `acceleration` - a normalized acceleration
    pub fn with_motion(mut self, velocity: Velocity, acceleration: f32) -> Self {
        self.velocity = velocity;
        self.acceleration = acceleration;
        self
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

    /// Updates the [Cursor], computing its velocity and acceleration
    /// # Arguments
    /// * `delta_time` - the [Duration] since last update
    /// * `position` - the new [Position]
    pub fn update(&mut self, delta_time: Duration, position: Position) {
        let delta_time = delta_time.as_secs_f32();
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
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.get_x_position() == other.get_x_position()
            && self.get_x_position() == other.get_y_position()
            && self.velocity == other.velocity
            && self.acceleration == other.acceleration
    }
}

#[cfg(test)]
mod tests {
    use std::{f32::consts::SQRT_2, time::Duration};

    use crate::cursor::{Cursor, Position};

    #[test]
    fn cursor_update() {
        let mut cursor = Cursor::new(0, Position { x: 0., y: 0. });

        cursor.update(Duration::from_secs(1), Position { x: 1., y: 1. });

        assert_eq!(cursor.get_x_position(), 1.);
        assert_eq!(cursor.get_y_position(), 1.);
        assert_eq!(cursor.get_x_velocity(), 1.);
        assert_eq!(cursor.get_y_velocity(), 1.);
        assert_eq!(cursor.get_acceleration(), SQRT_2);
    }
}
