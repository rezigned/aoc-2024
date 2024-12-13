use super::game::{
    avatar, is_obstacle, Game, GameStoreFields, GridStoreFields, Part, State, GUARD,
};
use leptos::prelude::*;
use leptos_use::{use_interval_fn, utils::Pausable};
use reactive_stores::Store;

// Speed and multiplers
const SPEED: u64 = 300;
const SPEED_FACTORS: [u64; 3] = [1, 2, 5];

#[component]
pub fn App() -> impl IntoView {
    let input = r"....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";

    let (speed, set_speed) = signal(SPEED / SPEED_FACTORS[0]);
    let game = Store::new(Game::new(input, Part::default()));

    let Pausable { pause, resume, .. } = use_interval_fn(
        move || {
            if game.state().get() == State::Running {
                game.write().update();

                // NOTE: Not sure if this is a bug in leptos's reactive store or leptos-use.
                // Without this line, the cells won't get updated.
                game.grid().cells().write();
            }
        },
        speed,
    );

    // Resume/pause the timer according to the current game state.
    Effect::new(move || {
        match game.state().get() {
            State::Ready | State::Paused | State::Done => pause(),
            State::Running => resume(),
        };
    });

    view! {
        <div class="grid">
            {move || game.grid().cells().get()
                .into_iter()
                .enumerate()
                .map(|(i, c)| {
                    let (x, y) = game.grid().get().to_position(i);
                    let is_guard = game.get().is_guard((x, y));
                    let symbol = cell_symbol(c, is_guard);
                    let classes = cell_classes(game, &(x, y), c, is_guard);

                    view! {
                        <div class=move || classes.join(" ")>{symbol}</div>
                    }
                })
                .collect_view()}
        </div>
        <div class="controls">
            <div class="btn-container">
                <button
                    class="play"
                    class:paused=move || game.get().is_running()
                    on:click=move |_| {
                        match game.state().get() {
                            State::Running => game.write().pause(),
                            State::Done => game.write().reset(),
                            _ => game.write().play(),
                        }
                    }
                />
            </div>
            <button
                class="button"
                class:selected=move || game.mode().get() == Part::One
                on:click=move |_| {
                    *game.write() = Game::new(input, Part::One);
                }
            >PART 1</button>
            <button
                class="button"
                class:selected=move || game.mode().get() == Part::Two
                on:click=move |_| {
                    *game.write() = Game::new(input, Part::Two);
                }
            >PART 2</button>
        </div>
        <div class="stats">
            <div class="col">
                <div>"speed: "
                {SPEED_FACTORS.map(|n| {
                    view! {
                        <a href="#"
                            class:selected=move || speed.get() == SPEED / n
                            on:click=move |e| {
                                e.prevent_default();
                                set_speed.set(SPEED / n);
                            }
                        >{n}</a>{move || if Some(&n) != SPEED_FACTORS.last() {" / "} else { "" }}
                    }
                }).collect_view()}</div>
            </div>
            <div>
                "visits: "<span class="num">{move || game.get().total_visits()}</span>
                <span class="sep">" | "</span>
                "loops: "<span class="num">{move || game.get().total_loops()}</span>
            </div>
        </div>
        <div class="credit"><a href="https://github.com/rezigned">@rezigned</a></div>
    }
}

fn cell_symbol(char: char, is_guard: bool) -> &'static str {
    if is_guard {
        avatar(GUARD)
    } else {
        avatar(if char == GUARD { ' ' } else { char })
    }
}

fn cell_classes<'a>(
    game: Store<Game>,
    position: &(i8, i8),
    char: char,
    is_guard: bool,
) -> Vec<&'a str> {
    let mut classes = vec!["cell"];

    // build dynamic css classes for cell
    if is_obstacle(char) {
        classes.push("obstacle")
    }
    if game.get().is_visited(position) {
        classes.push("visited")
    }
    if is_guard {
        classes.push("arrow");
        classes.push(game.direction().get().as_str());
    }

    classes
}
