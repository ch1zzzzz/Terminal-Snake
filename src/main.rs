use rand::random;
use crossterm::{
    execute,
    cursor::{
        Hide,
        Show,
        MoveTo
    },
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
        SetTitle,
        size,
        Clear,
        ClearType
    },
    style::{
        SetBackgroundColor,
        SetForegroundColor,
        ResetColor,
        Color,
        Print
    },
    event::{
        read,
        poll,
        Event,
        KeyCode
    },
    Result
};
use std::{
    time::{
        Duration, 
        Instant
    },
    io::{
        stdout, 
        Stdout
    }
};

#[derive(Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn kind(&self) -> DirectionKind {
        match self {
            Direction::Up => DirectionKind::Vertical,
            Direction::Down => DirectionKind::Vertical,
            _ => DirectionKind::Horizontal
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum DirectionKind {
    Horizontal,
    Vertical
}

#[derive(Clone, PartialEq, Eq)]
struct SnakeGameCord {
    x: usize,
    y: usize
}

impl SnakeGameCord {
    fn move_direction(&mut self, direction: &Direction) {
        match direction {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1
        }
    }

    fn moved_direction(&self, direction: &Direction) -> SnakeGameCord {
        let mut new_cord = self.clone();

        new_cord.move_direction(direction);

        new_cord
    }
}

struct SnakeGame {
    data: Vec<Vec<i32>>,
    snake_head_pos: SnakeGameCord,
    snake_len: u32,
    snake_saturation_len: u32,
    grew_last_tick: bool,
    dead: bool,
    game_grow_rate: u32,
    max_apple_count: u32,
    min_apple_count: u32,
    ticks_between_apple_spawn: u32,
    ticks_since_last_apple_spawned: u32
}

impl SnakeGame {
    fn create(width: usize, height: usize, game_grow_rate: u32, max_apple_count: u32, min_apple_count: u32, ticks_between_apple_spawn: u32) -> SnakeGame {
        let row = vec![0].repeat(width);

        let mut data = vec![];

        for _ in 0..height {
            data.push(row.clone());
        }

        SnakeGame { 
            data, 
            snake_head_pos: SnakeGameCord { x: width/3, y: height/2 }, 
            snake_len: 0,
            snake_saturation_len: 3,
            grew_last_tick: true,
            dead: false,
            game_grow_rate,
            max_apple_count,
            min_apple_count,
            ticks_between_apple_spawn,
            ticks_since_last_apple_spawned: 0
        }
    }

    fn tick(&mut self, direction: &Direction) {
        /*
            1. Is move in bounds
            2. Is move into snake
            3. Is move into apple
            4. Shorten tail
            5. Put head
            6. Respawn apple
        */

        // is move in bounds
        if self.would_move_out_of_bounds(direction) {
            self.dead = true;
            return;
        }

        let new_head_pos = self.snake_head_pos.moved_direction(direction);

        let value_at_new_head_pos = self.data[new_head_pos.y][new_head_pos.x];

        // is move into body
        if value_at_new_head_pos > 0 {
            self.dead = true;
            return;
        }

        // is move into apple
        if value_at_new_head_pos == -1 {
            self.snake_saturation_len += self.game_grow_rate;
        }

        // shorten tail
        if self.snake_len < self.snake_saturation_len && ! self.grew_last_tick {
            self.snake_len += 1;
            self.grew_last_tick = true;
        } else {
            self.shorten_snake();
            self.grew_last_tick = false;
        }

        // put head
        self.data[new_head_pos.y][new_head_pos.x] = self.snake_len as i32;
        self.snake_head_pos = new_head_pos;

        // respawn apple
        if self.count_apples() < self.max_apple_count {
            self.ticks_since_last_apple_spawned += 1;

            'apple_spawning: while self.ticks_since_last_apple_spawned > self.ticks_between_apple_spawn || self.count_apples() < self.min_apple_count {
                self.ticks_since_last_apple_spawned = 0;

                if ! self.spawn_apple() {
                    break 'apple_spawning;
                }
            }
        }

    }

    fn would_move_out_of_bounds(&self, direction: &Direction) -> bool {
        match direction {
            Direction::Up => if self.snake_head_pos.y == 0 {
                return true
            },
            Direction::Down => if self.snake_head_pos.y + 1 == self.data.len() {
                return true
            },
            Direction::Left => if self.snake_head_pos.x == 0 {
                return true
            },
            Direction::Right => if self.snake_head_pos.x + 1 == self.data[0].len() {
                return true
            }
        }
        
        false
    }

    fn shorten_snake(&mut self) {
        for row in &mut self.data {
            for col in row {
                if *col > 0 {
                    *col -= 1;
                }
            }
        }
    }

    fn count_apples(&self) -> u32 {
        let mut counter : u32 = 0;

        for row in &self.data {
            for col in row {
                if *col == -1 {
                    counter += 1;
                }
            }
        }

        counter
    }

    fn count_free(&self) -> u32 {
        let mut counter : u32 = 0;

        for row in &self.data {
            for col in row {
                if *col == 0 {
                    counter += 1;
                }
            }
        }

        counter
    }

    fn spawn_apple(&mut self) -> bool {
        let free = self.count_free();
        let pos = random::<u32>() % free;

        let mut count = 1;

        for row in &mut self.data {
            for col in row {
                if *col == 0 {
                    if count == pos {
                        *col = -1;
                        return true;
                    }
                    count += 1;
                }
            }
        }

        false
    }

    fn clear(&mut self) {
        for row in &mut self.data {
            for col in row {
                *col = 0;
            }
        }

        self.snake_head_pos = SnakeGameCord { x: self.data[0].len() / 3, y: self.data.len() / 2 };
        self.snake_len = 0;
        self.snake_saturation_len = 3;
        self.grew_last_tick = true;
        self.dead = false;
    }
}

fn calculate_margins(width: u16, height: u16) -> Result<(u16, u16)> {
    let (s_width, s_height) = size()?;

    let width = if width + 2 > s_width {
        s_width - 2
    } else {
        width
    };
    let height = if height + 2 > s_height {
        s_height - 2
    } else {
        height
    };

    Ok(((s_width - width - 2) / 2, (s_height - height - 2) / 2))
}

fn display_game(stdout: &mut Stdout, game: &SnakeGame) -> Result<()> {
    let (margin_left, margin_top) = calculate_margins(game.data[0].len() as u16 * 2, game.data.len() as u16)?;
    
    execute!(stdout, MoveTo(margin_left, margin_top), Print(String::from("#").repeat(game.data[0].len() * 2 + 2)))?;

    for (y, row) in game.data.iter().enumerate() {
        execute!(stdout, MoveTo(margin_left, y as u16 + 1 + margin_top), Print("#"))?;

        for col in row {
            match *col {
                -1 => execute!(stdout, SetForegroundColor(Color::Red), Print("()"), SetForegroundColor(Color::Grey))?,
                0 => execute!(stdout, Print("  "))?,
                _ => execute!(stdout, SetForegroundColor(Color::Green), Print("[]"), SetForegroundColor(Color::Grey))?
            }
        }

        execute!(stdout, Print("#"))?;
    }

    execute!(stdout, MoveTo(margin_left, game.data.len() as u16 + 1 + margin_top), Print(String::from("#").repeat(game.data[0].len() * 2 + 2)))?;

    Ok(())
}



// the width and height are the inner width and height
fn menue(stdout: &mut Stdout, width: u16, height: u16, title: Option<&str>, items: &[&str]) -> Result<usize> {
    let mut selected = 0;

    execute!(stdout, Clear(ClearType::All))?;
    
    loop {
        let (margin_left, mut margin_top) = calculate_margins(width, height)?;
        let mut height = height;

        // lines
        let bar = String::from("#").repeat((width + 2) as usize);
        let inner_bar = format!("#{}#", String::from(" ").repeat(width as usize));

        //draw square
        execute!(
            stdout,
            MoveTo(margin_left, margin_top),
            Print(&bar),
            MoveTo(margin_left, margin_top + height + 1),
            Print(bar),
        )?;

        for y in 1..(height + 1) {
            execute!(stdout, MoveTo(margin_left, margin_top + y), Print(&inner_bar))?;
        }

        // title
        match &title {
            Some(title_text) => {
                execute!(
                    stdout, 
                    MoveTo(margin_left + (width - title_text.len() as u16) / 2 + 1, margin_top + 1), 
                    SetForegroundColor(Color::Green), 
                    Print(title_text), 
                    SetForegroundColor(Color::White)
                )?;
                height -= 1;
                margin_top += 1;
            },
            _ => {}
        }

        // draw entries
        let spacing = (height as usize - items.len()) / (items.len() + 1);

        for (i, item) in items.iter().enumerate() {
            execute!(stdout, MoveTo(margin_left + (width - item.len() as u16) / 2 + 1, margin_top + ((i as u16 + 1) * (spacing as u16)) + 1 + i as u16))?;

            if i == selected {
                execute!(stdout, SetBackgroundColor(Color::White), SetForegroundColor(Color::Black))?;
            }

            execute!(stdout, Print(item), SetBackgroundColor(Color::Black), SetForegroundColor(Color::White))?;
        }

        // process events
        match read()? {
            Event::Key(keyevent) => match keyevent.code {
                KeyCode::Down => if selected < items.len() - 1 {
                    selected += 1;
                },
                KeyCode::Up => if selected > 0 {
                    selected -= 1;
                },
                KeyCode::Enter => {
                    return Ok(selected);
                },
                _ => {}
            },
            _ => {}
        }
    }
}

fn play_game(stdout: &mut Stdout, game: &mut SnakeGame, steps_per_second: u32) -> Result<()> {
    let millis_delay = 1000 / steps_per_second;
    
    'retry: loop {
        let mut direction = Direction::Right;

        let mut queue = vec![];

        execute!(stdout, Clear(ClearType::All))?;

        for _ in 0..3 {
            game.tick(&direction);
        }

        while ! game.dead {
            let now = Instant::now();

            while poll(Duration::from_secs(0))? {
                
                match read()? {
                    Event::Key(key_event) => {
                        match key_event.code {
                            KeyCode::Up => queue.push(Direction::Up),
                            KeyCode::Down => queue.push(Direction::Down),
                            KeyCode::Left => queue.push(Direction::Left),
                            KeyCode::Right => queue.push(Direction::Right),
                            KeyCode::Esc => match menue(stdout, 32, 9, None, &["CONTINUE", "EXIT"])? {
                                0 => {},
                                1 => break 'retry,
                                _ => {}
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }

            let mut iter = queue.iter();

            'select_next_relevant_queue_result: loop {
                match iter.next() {
                    Some(dir) => if dir.kind() != direction.kind() {
                        direction = dir.clone();
                        break 'select_next_relevant_queue_result;
                    },
                    None => {
                        break 'select_next_relevant_queue_result;
                    }
                }
            }

            let mut new_queue = vec![];

            'collect_remaining: loop {
                match iter.next() {
                    Some(dir) => new_queue.push(dir.clone()),
                    None => {
                        break 'collect_remaining;
                    }
                }
            }

            queue = new_queue;

            game.tick(&direction);
            display_game(stdout, &game)?;

            while now.elapsed() < Duration::from_millis(millis_delay as u64) {}
        }

        'viewing: loop {
            match menue(stdout, 32, 8, Some(format!("You got to a length of {}", game.snake_len).as_str()), &["RETRY", "VIEW", "EXIT"])? {
                0 => {
                    game.clear();
                    break 'viewing;
                },
                1 => {
                    display_game(stdout, game)?;
                    wait_for_any_key_press()?;
                },
                _ => break 'retry
            }
        }
    }

    Ok(())
}

fn wait_for_any_key_press() -> Result<()> {
    'wait: loop {
        match read()? {
            Event::Key(_) => break 'wait,
            _ => {}
        }
    }

    Ok(())
}

fn display_box(stdout: &mut Stdout, width: u16, height: u16, margin_left: u16, margin_top: u16) -> Result<()> {
    let bar = String::from("#").repeat((width + 2) as usize);
    let inner_bar = format!("#{}#", String::from(" ").repeat(width as usize));

    execute!(
        stdout,
        MoveTo(margin_left, margin_top),
        Print(&bar),
        MoveTo(margin_left, margin_top + height + 1),
        Print(bar)
    )?;

    for x in 1..height+1 {
        execute!(
            stdout,
            MoveTo(margin_left, margin_top+x),
            Print(&inner_bar)
        )?;
    }

    Ok(())
}

fn message_box(stdout: &mut Stdout, width: u16, height: u16, text: String) -> Result<()> {
    let (margin_left, margin_top) = calculate_margins(width, height)?;

    display_box(stdout, width, height, margin_left, margin_top)?;

    let lines : Vec<&str> = text.split('\n').collect();

    let spacing = if lines.len() > height as usize {
        0
    } else {
        ((height as usize - lines.len()) / 2) as u16
    };

    for (i, line) in lines.iter().enumerate() {
        execute!(
            stdout,
            MoveTo(margin_left as u16 + if line.len() > width as usize {
                0
            } else {
                (width - line.len() as u16) / 2
            }, margin_top + spacing + i as u16),
            Print(line)
        )?;
    }

    Ok(())
}

fn set_size(stdout: &mut Stdout, width: &mut usize, height: &mut usize) -> Result<()> {
    let txt = 
    r"Use the Arrow keys 
    to
    change the 
    dimensions of the game.
    Press ENTER when you are done";

    'sellect: loop {
        message_box(stdout, *width as u16, *height as u16, String::from(txt))?;

        match read()? {
            Event::Key(keyevent) => match keyevent.code {
                KeyCode::Up => *height += 1,
                KeyCode::Down => *height -= 1,
                KeyCode::Left => *width -= 1,
                KeyCode::Right => *width += 1,
                KeyCode::Enter => break 'sellect,
                _ => {}
            },
            _ => {}
        }

        if *width < 10 {
            *width = 10;
        }
        if *height < 10 {
            *height = 10;
        }
    }

    Ok(())
}

fn request_number(stdout: &mut Stdout) -> Result<u32> {
    let mut numb_str = String::from("");

    loop {
        let txt = format!("Enter number: {}_", numb_str);

        message_box(stdout, (txt.len() + 6) as u16, 5, txt)?;

        match read()? {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(c) => if c.is_numeric() {
                    numb_str.push(c);
                },
                KeyCode::Enter => {
                    match numb_str.parse() {
                        Ok(numb) => return Ok(numb),
                        Err(_) => {
                            let txt = format!("{} isn't a valid number", numb_str);

                            message_box(stdout, (txt.len() + 6) as u16, 5, txt)?;
                        }
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }
}

fn set_apple_settings(stdout: &mut Stdout, min_apple_count: &mut u32, max_apple_count: &mut u32, ticks_between_apple_spawn: &mut u32) -> Result<()> {
    'in_menue: loop {
        match menue(stdout, 80, 20, Some("Apple Settings"), &[
            format!("MIN: {}", min_apple_count).as_str(), 
            format!("MAX: {}", max_apple_count).as_str(),
            format!("TICKS INBETWEEN SPAWNS: {}", ticks_between_apple_spawn).as_str(),
            "DONE"
        ])? {
            0 => {
                *min_apple_count = request_number(stdout)?;

                if min_apple_count > max_apple_count {
                    *max_apple_count = *min_apple_count;
                }
            },
            1 => {
                *max_apple_count = request_number(stdout)?;

                if max_apple_count < min_apple_count {
                    *min_apple_count = *max_apple_count;
                }
            },
            2 => *ticks_between_apple_spawn = request_number(stdout)?,
            _ => break 'in_menue
        }
    }

    Ok(())
}

fn set_snake_settings(stdout: &mut Stdout, game_grow_rate: &mut u32, steps_per_second: &mut u32) -> Result<()> {
    'settings: loop {
        match menue(stdout, 80, 20, Some("Snake Settings"), &[
            format!("STEPS PER SECOND: {}", steps_per_second).as_str(), 
            format!("GROWTH PER APPLE: {}", game_grow_rate).as_str(), 
            "DONE"
        ])? {
            0 => *steps_per_second = request_number(stdout)?,
            1 => *game_grow_rate = request_number(stdout)?,
            _ => break 'settings
        }

        if *steps_per_second == 0 {
            *steps_per_second = 1;
        }

        if *game_grow_rate == 0 {
            *game_grow_rate = 1;
        }
    }

    Ok(())
}

enum DisplayMode {
    Slim,
    Blocky
}

fn main() -> Result<()> {
    let mut stdout = stdout();

    execute!(stdout, Hide, SetBackgroundColor(Color::Black), SetForegroundColor(Color::White), EnterAlternateScreen, SetTitle("Terminal Snake"))?;

    enable_raw_mode()?;

    let mut width: usize = 80;
    let mut height: usize = 30;
    let mut min_apple_count: u32 = 1;
    let mut max_apple_count: u32 = 1;
    let mut ticks_between_apple_spawn: u32 = 100;
    let mut game_grow_rate: u32 = 1;
    let mut steps_per_second: u32 = 10;

    let mut display_mode = DisplayMode::Blocky;

    'application: loop {
        'selection: loop {     
            let mut game = SnakeGame::create(width / 2, height, game_grow_rate, max_apple_count, min_apple_count, ticks_between_apple_spawn);
            
            match menue(&mut stdout, 80, 20, Some("T E R M I N A L   S N A K E"), &["PLAY", "SETTINGS", "QUIT"])? {
                0 => play_game(&mut stdout, &mut game, steps_per_second)?,
                1 => loop {
                    match menue(&mut stdout, 80, 20, Some("SETTINGS"), &[
                        "SIZE",
                        "APPLES",
                        "SNAKE",
                        "BACK"
                    ])? {
                        0 => set_size(&mut stdout, &mut width, &mut height)?,
                        1 => set_apple_settings(&mut stdout, &mut min_apple_count, &mut max_apple_count, &mut ticks_between_apple_spawn)?,
                        2 => set_snake_settings(&mut stdout, &mut game_grow_rate, &mut steps_per_second)?,
                        _ => break 'selection
                    }
                }
                _ => break 'application
            }
        }
    }

    disable_raw_mode()?;

    execute!(stdout, Show, ResetColor, LeaveAlternateScreen)?;

    Ok(())
}

/* 
    TODO
    - some things are settings (size)
    - more settings (easy, borderless)
    - save highscores (specific to mode)
    - look at highscores in the main menue
    - display mode during play
    
    - windowed version
    - upload to itch.io

    - check compatibility

    Done
    - grapic update("[]", "()" and light gray borders)
    - set size
    - gamemodes
    - score
    - retry
    - color
    - esc & pause menue
    - adapt to screen size
*/
