use std::time::Duration;

use crate::osc_encode_decode::CursorParams;

// #[derive(Clone, Copy)]
#[derive(Default, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn distance_from(&self, point: &Point) -> f32 {
        let dx = self.x - point.x;
        let dy = self.y - point.y;
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Added,
    Accelerating,
    Decelerating,
    Rotating,
    Stopped,
    Removed,
}

// #[derive(Clone, Copy)]
#[derive(Debug)]
pub struct Cursor {
    session_id: i32,
    time: Duration,
    path: Vec<Point>,
    velocity: Velocity,
    acceleration: f32,
    state: State,
}

impl Cursor {
    pub fn new(time: Duration, session_id: i32 /*, source: Source*/, position: Point) -> Self {
        Self {
            session_id,
            path: Vec::from([position]),
            velocity: Velocity::default(),
            acceleration: 0f32,
            time,
            state: State::Added,
        }
    }

    pub fn with_movement(mut self, velocity: Velocity, acceleration: f32) -> Self {
        self.velocity = velocity;
        self.acceleration = acceleration;
        self
    }

    pub fn get_session_id(&self) -> i32 {
        self.session_id
    }

    pub fn get_time(&self) -> Duration {
        self.time
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

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn update(&mut self, time: Duration, position: Point) {
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
        println!("({speed} - {last_speed}) / {delta_time}");
        self.acceleration = (speed - last_speed) / delta_time;
        self.path.push(position);
        self.time = time;

        self.state = if self.acceleration > 0f32 {
            State::Accelerating
        } else if self.acceleration < 0f32 {
            State::Decelerating
        } else {
            State::Stopped
        };
    }

    pub fn update_values(
        &mut self,
        time: Duration,
        position: Point,
        velocity: Velocity,
        acceleration: f32,
    ) {
        self.time = time;
        self.path.push(position);
        self.velocity = velocity;
        self.acceleration = acceleration;
    }

    pub fn update_from_params(&mut self, time: Duration, params: CursorParams) {
        self.time = time;
        self.path.push(Point{x: params.x_pos, y: params.y_pos});
        self.velocity = Velocity{x: params.x_vel, y: params.y_vel};
        self.acceleration = params.acceleration;
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

impl From<(Duration, CursorParams)> for Cursor {
    fn from((time, params): (Duration, CursorParams)) -> Self {
        Self {
            session_id: params.session_id,
            path: vec![Point{x: params.x_pos, y: params.y_pos}],
            velocity: Velocity{x: params.x_vel, y: params.y_vel},
            acceleration: params.acceleration,
            time,
            state: State::Added,
        }
    }
}

impl From<CursorParams> for Cursor {
    fn from(params: CursorParams) -> Self {
        (Duration::default(), params).into()
    }
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, f32::consts::SQRT_2};

    use crate::cursor::{Cursor, Point};

    #[test]
    fn cursor_update() {
        let mut cursor = Cursor::new(Duration::default(), 0, Point { x: 0., y: 0. });

        cursor.update(Duration::from_secs(1), Point { x: 1., y: 1. });

        assert_eq!(cursor.get_x_position(), 1.);
        assert_eq!(cursor.get_y_position(), 1.);
        assert_eq!(cursor.get_x_velocity(), 1.);
        assert_eq!(cursor.get_y_velocity(), 1.);
        assert_eq!(cursor.get_acceleration(), SQRT_2);
    }
}
