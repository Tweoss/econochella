use rand::prelude::*;

const TEMPERATURE: f64 = 0.5;

#[derive(Clone)]
struct Econochella {
    tent: Venue,
    amphitheater: Venue,
    stadium: Venue,
    /// a possible band "card" that can be used, and the corresponding current location
    knapsack: Vec<(Band, Location)>,
    /// total budget
    budget: u32,
    /// temperature controls tradeoff between exploration and exploitation
    temperature: f64,
}

impl Econochella {
    fn new(budget: u32, temperature: f64, bands: Vec<Band>) -> Econochella {
        let knapsack = bands
            .iter()
            .map(|band| (band.clone(), Location::Unused))
            .collect();
        Econochella {
            tent: Venue::new("tent".to_string(), 15, 300),
            amphitheater: Venue::new("amphitheater".to_string(), 30, 360),
            stadium: Venue::new("stadium".to_string(), 30, 360),
            knapsack,
            budget,
            temperature,
        }
    }
    fn choose_band(&mut self, rng: &mut ThreadRng) -> (usize, Location) {
        let len = self.knapsack.len();
        let index = rng.gen_range(0..len);
        (index, self.knapsack[index].1.clone())
    }
    fn move_band(
        &mut self,
        band_index: usize,
        original_location: Location,
        location: Location,
        rng: &mut ThreadRng,
    ) {
        self.knapsack[band_index].1 = location.clone();
        let band = &self.knapsack[band_index].0;
        match original_location {
            Location::Tent => self.tent.remove_band(band),
            Location::Amphitheater => self.amphitheater.remove_band(band),
            Location::Stadium => self.stadium.remove_band(band),
            Location::Unused => (),
        }
        match location {
            Location::Tent => self.tent.add_band(band.clone(), rng),
            Location::Amphitheater => self.amphitheater.add_band(band.clone(), rng),
            Location::Stadium => self.stadium.add_band(band.clone(), rng),
            Location::Unused => (),
        }
    }
    fn valid(&self) -> bool {
        // check the budget
        if self.budget < self.amphitheater.cost() + self.stadium.cost() + self.tent.cost() {
            return false;
        }
        // check the time
        if self.tent.time() > self.tent.total_time
            || self.amphitheater.time() > self.amphitheater.total_time
            || self.stadium.time() > self.stadium.total_time
        {
            return false;
        }
        // check special conditions
        if !self.special_conditions() {
            return false;
        }
        true
    }
    /// check's several special conditions
    fn special_conditions(&self) -> bool {
        let macy_dynamite = {
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "Macy Dynamite")
                .unwrap()
                .clone()
                .1;
            if let Some(TimeSlot::BandSlot(start_time, _)) =
                self.get_schedule(&loc).iter().find(|slot| {
                    if let TimeSlot::BandSlot(_, band) = slot {
                        &band.name == "Macy Dynamite"
                    } else {
                        false
                    }
                })
            {
                // must play after 9 pm if playing
                time_is_after(9 * 60, *start_time, loc)
            } else {
                // otherwise tru
                true
            }
        };
        let illiterate_monkeys = {
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "Illiterate Monkeys")
                .unwrap()
                .clone()
                .1;
            !self.any_same_stage(&loc, vec!["Fractured Coccyx", "Macaulay & Co."])
        };
        let onyx_eyes = {
            // chicken fried awesome must also be playing
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "Onyx Eyes")
                .unwrap()
                .clone()
                .1;
            if loc == Location::Unused {
                true
            } else {
                // chicken fried awesome must be booked in some venue
                self.amphitheater.schedule.iter().any(|a| {
                    if let TimeSlot::BandSlot(_, band) = a {
                        &band.name == "Chicken Fried Awesome"
                    } else {
                        false
                    }
                }) || self.stadium.schedule.iter().any(|a| {
                    if let TimeSlot::BandSlot(_, band) = a {
                        &band.name == "Chicken Fried Awesome"
                    } else {
                        false
                    }
                }) || self.tent.schedule.iter().any(|a| {
                    if let TimeSlot::BandSlot(_, band) = a {
                        &band.name == "Chicken Fried Awesome"
                    } else {
                        false
                    }
                })
            }
        };
        let babes_and_bullets = {
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "Babes and Bullets")
                .unwrap()
                .clone()
                .1;
            !self.any_same_stage(
                &loc,
                vec!["Rico’s Revenge", "Robert Miyagi", "DJ Swedissh Cheff"],
            )
        };
        let infusion = {
            // must be playing back to back
            let first_pos = self
                .knapsack
                .iter()
                .position(|(band, _)| &band.name == "Infu$ion")
                .unwrap();
            let first_loc = self.knapsack[first_pos].1.clone();
            let second_loc = self
                .knapsack
                .iter()
                .skip(first_pos)
                .find(|(band, _)| &band.name == "Infu$ion")
                .unwrap()
                .clone()
                .1;
            if first_loc == second_loc && first_loc != Location::Unused {
                let schedule = match first_loc {
                    Location::Amphitheater => &self.amphitheater.schedule,
                    Location::Stadium => &self.stadium.schedule,
                    Location::Tent => &self.tent.schedule,
                    Location::Unused => panic!("unused"),
                };
                let first = schedule
                    .iter()
                    .position(|slot| {
                        if let TimeSlot::BandSlot(_, band) = slot {
                            &band.name == "Infu$ion"
                        } else {
                            false
                        }
                    })
                    .unwrap();
                let second = schedule
                    .iter()
                    .position(|slot| {
                        if let TimeSlot::BandSlot(_, band) = slot {
                            &band.name == "Infu$ion"
                        } else {
                            false
                        }
                    })
                    .unwrap();
                if first > second {
                    first - second == 1
                } else {
                    second - first == 1
                }
            } else {
                true
            }
        };
        let hummingbird_anthem = {
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "Hummingbird Anthem")
                .unwrap()
                .clone()
                .1;
            if let Some(TimeSlot::BandSlot(start_time, _)) =
                self.get_schedule(&loc).iter().find(|slot| {
                    if let TimeSlot::BandSlot(_, band) = slot {
                        &band.name == "Hummingbird Anthem"
                    } else {
                        false
                    }
                })
            {
                // must play before 9 pm if playing
                !time_is_after(9 * 60, *start_time, loc)
            } else {
                // otherwise true
                true
            }
        };
        let macaulay_and_co = {
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "Macaulay & Co.")
                .unwrap()
                .clone()
                .1;
            if loc == Location::Unused {
                true
            } else {
                let schedule = match loc {
                    Location::Amphitheater => &self.amphitheater.schedule,
                    Location::Stadium => &self.stadium.schedule,
                    Location::Tent => &self.tent.schedule,
                    Location::Unused => panic!("unused"),
                };
                if let Some(TimeSlot::BandSlot(_, band)) = schedule.last() {
                    &band.name == "Macaulay & Co."
                } else {
                    false
                }
            }
        };

        let dj_megara = {
            // cannot have the other dj before or after
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "DJ Megara")
                .unwrap()
                .clone()
                .1;
            if loc == Location::Unused {
                true
            } else {
                let schedule = self.get_schedule(&loc);
                let pos = schedule
                    .iter()
                    .position(|slot| {
                        if let TimeSlot::BandSlot(_, band) = slot {
                            &band.name == "DJ Megara"
                        } else {
                            false
                        }
                    })
                    .unwrap();
                match (
                    schedule.get(pos.checked_sub(1).unwrap_or(usize::MAX)),
                    schedule.get(pos + 1),
                ) {
                    (Some(TimeSlot::BandSlot(_, band)), _) if band.name == "DJ Swedissh Cheff" => {
                        false
                    }
                    (_, Some(TimeSlot::BandSlot(_, band))) if band.name == "DJ Swedissh Cheff" => {
                        false
                    }
                    _ => true,
                }
            }
        };

        let fractured_coccyx = {
            // cannot play in the tent
            let loc = self
                .knapsack
                .iter()
                .find(|(band, _)| &band.name == "Fractured Coccyx")
                .unwrap()
                .clone()
                .1;
            loc != Location::Tent
        };

        macy_dynamite
            && illiterate_monkeys
            && onyx_eyes
            && babes_and_bullets
            && infusion
            && hummingbird_anthem
            && macaulay_and_co
            && dj_megara
            && fractured_coccyx
    }
    fn value(&self) -> u32 {
        self.amphitheater.value()
            + self.stadium.value()
            + self.tent.value()
            + self.special_bonuses()
    }
    fn special_bonuses(&self) -> u32 {
        0
    }
    fn get_schedule(&self, loc: &Location) -> Vec<TimeSlot> {
        match loc {
            Location::Tent => self.tent.schedule.clone(),
            Location::Amphitheater => self.amphitheater.schedule.clone(),
            Location::Stadium => self.stadium.schedule.clone(),
            Location::Unused => Vec::new(),
        }
    }
    /// returns true if any band from other names is playing in this location
    fn any_same_stage(&self, loc: &Location, other_names: Vec<&str>) -> bool {
        let schedule = self.get_schedule(loc);
        // check for each name if the scheule contains this band
        other_names.iter().any(|other_name| {
            schedule.iter().any(|a| {
                if let TimeSlot::BandSlot(_, band) = a {
                    &band.name == other_name
                } else {
                    false
                }
            })
        })
    }
}

#[derive(Clone)]
struct Venue {
    /// the venue's name
    name: String,
    /// The schedule, consisting of time slots of either a band or a break.
    schedule: Vec<TimeSlot>,
    /// Current time from start in minutes
    current_time: u32,
    /// Total time from start in minutes
    total_time: u32,
    /// Standard break time in minutes for this venue
    break_time: u32,
}

impl Venue {
    fn new(name: String, break_time: u32, total_time: u32) -> Venue {
        Venue {
            name,
            schedule: Vec::new(),
            current_time: 0,
            total_time,
            break_time,
        }
    }
    /// remove the last band with this name from the schedule
    fn remove_band(&mut self, band: &Band) {
        // find the last band with this name
        if let Some(index) = self.schedule.iter().rposition(|time_slot| {
            if let TimeSlot::BandSlot(_, b) = time_slot {
                if b.name == band.name {
                    return true;
                }
            };
            false
        }) {
            if self.schedule.len() == 1 {
                self.schedule.clear();
                return;
            }
            if index == 0 {
                // remove first two
                self.schedule.drain(0..2);
                return;
            }
            // remove and keep the value of the band
            let slot = self.schedule.drain(index - 1..=index).next().unwrap();
            // shift the remaining band's times
            if let TimeSlot::BandSlot(start_time, band) = slot {
                self.current_time -= band.time;
                self.current_time -= self.break_time;
                if let Some(TimeSlot::BandSlot(next_start_time, _)) = self.schedule.get(index) {
                    let shift = next_start_time - start_time;
                    self.schedule.iter_mut().skip(index).for_each(|time_slot| {
                        if let TimeSlot::BandSlot(time, _) = time_slot {
                            *time -= shift;
                        }
                    });
                }
            }
        }
    }
    /// add a band to the schedule in a random place
    fn add_band(&mut self, band: Band, rng: &mut ThreadRng) {
        if self.schedule.is_empty() {
            self.current_time += band.time;
            self.schedule.push(TimeSlot::BandSlot(0, band));
            return;
        }
        // insert a break, then increment the current time
        let index = rng.gen_range(0..=(self.schedule.len() + 1) / 2) * 2;
        if index == self.schedule.len() + 1 {
            // insert a break after the last band and then the band
            self.schedule.push(TimeSlot::Break);
            self.current_time += self.break_time;
            let temp_time = band.time;
            self.schedule
                .push(TimeSlot::BandSlot(self.current_time, band));
            self.current_time += temp_time;
        } else {
            // insert band then a break after it
            let time = if let TimeSlot::BandSlot(t, _) = self.schedule[index] {
                t
            } else {
                panic!("expected band slot");
            };
            let temp_time = band.time;
            self.schedule.insert(index, TimeSlot::BandSlot(time, band));
            self.current_time += temp_time;
            self.schedule.insert(index + 1, TimeSlot::Break);
            self.current_time += self.break_time;
        }
    }
    /// find the total cost of the schedule
    fn cost(&self) -> u32 {
        self.schedule
            .iter()
            .fold(0, |acc, time_slot| match time_slot {
                TimeSlot::BandSlot(_, band) => acc + band.cost,
                TimeSlot::Break => acc,
            })
    }
    /// find the total time of the schedule
    fn time(&self) -> u32 {
        self.schedule
            .iter()
            .fold(0, |acc, time_slot| match time_slot {
                TimeSlot::BandSlot(_, band) => acc + band.time,
                TimeSlot::Break => acc + self.break_time,
            })
    }

    /// find the value of the schedule
    fn value(&self) -> u32 {
        self.schedule
            .iter()
            .fold(0, |acc, time_slot| match time_slot {
                TimeSlot::BandSlot(_, band) => acc + band.value(),
                TimeSlot::Break => acc,
            })
    }
}

#[derive(Clone, PartialEq)]
enum Location {
    Tent,
    Amphitheater,
    Stadium,
    Unused,
}

impl Location {
    fn choose_location(&self, rng: &mut ThreadRng) -> Location {
        match rng.gen_range(0..4) {
            0 => Location::Tent,
            1 => Location::Amphitheater,
            2 => Location::Stadium,
            3 => Location::Unused,
            _ => panic!("invalid location"),
        }
    }
}

#[derive(Clone, Debug)]
enum TimeSlot {
    Break,
    /// (time of starting, band)
    BandSlot(u32, Band),
}

#[derive(Clone, Debug)]
struct Band {
    /// Name of band
    name: String,
    /// Time to play in minutes
    time: u32,
    /// Anticipated revenue in dollars
    /// (if this were not accounted for, we would not have econochella at all. we would simply pocket the budget)
    revenue: u32,
    /// Cost in dollars
    cost: u32,
}

impl Band {
    /// find the value of the band
    fn value(&self) -> u32 {
        self.revenue - self.cost
    }
}

/// Expected time ought to be the minutes since noon
fn time_is_after(expected_time: u32, time_since_start: u32, loc: Location) -> bool {
    let start_time = match loc {
        Location::Tent => 300,
        Location::Amphitheater => 240,
        Location::Stadium => 360,
        Location::Unused => u32::MIN,
    };
    start_time + time_since_start > expected_time
}

/// Initialize the Econochella, Venues, and Bands
/// Econochella has the best seen knapsack, we operate on a running subset
/// For the number of iterations
///     Create a temporary knapsack / Econochella
///     Choose a band in the temporary knapsack / Econochella
///     Randomly choose a different state (none or location) and change the band's state
///     If this is feasible // ! run the checking function
///         delta = the score of the temporary knapsack / Econochella - the score of the running Econochella
///         If the delta is positive OR if a randum number between 0 and 1 < e ^ (delta / Temperature)
///             update the running best knapsack in Econochella
///         If the score of the running knapsack is better than the score of the best knapsack / Econochella
///             update the best knapsack in Econochella
fn main() {
    let knapsack: Vec<Band> = vec![
        Band {
            name: "The Bionic Men".to_string(),
            time: 60,
            revenue: 300_000,
            cost: 100_000,
        },
        Band {
            name: "Les Salter and the Ignition".to_string(),
            time: 80,
            revenue: 300_000,
            cost: 95_000,
        },
        Band {
            name: "Macy Dynamite".to_string(),
            time: 60,
            revenue: 900_000,
            cost: 270_000,
        },
        Band {
            name: "Illiterate Monkeys".to_string(),
            time: 60,
            revenue: 200_000,
            cost: 75_000,
        },
        Band {
            name: "Chicken Fried Awesome".to_string(),
            time: 40,
            revenue: 75_000,
            cost: 25_000,
        },
        Band {
            name: "Babes and Bullets".to_string(),
            time: 40,
            revenue: 200_000,
            cost: 40_000,
        },
        Band {
            name: "Infu$ion".to_string(),
            time: 50,
            revenue: 100_000,
            cost: 65_000,
        },
        Band {
            name: "Infu$ion".to_string(),
            time: 50,
            revenue: 100_000,
            cost: 65_000,
        },
        Band {
            name: "Rico’s Revenge".to_string(),
            time: 70,
            revenue: 500_000,
            cost: 300_000,
        },
        Band {
            name: "The Potato Head Project".to_string(),
            time: 30,
            revenue: 200_000,
            cost: 18_000,
        },
        Band {
            name: "Robert Miyagi".to_string(),
            time: 90,
            revenue: 600_000,
            cost: 340_000,
        },
        Band {
            name: "Aluminum Falcon".to_string(),
            time: 60,
            revenue: 300_000,
            cost: 45_000,
        },
        Band {
            name: "DJ Swedissh Cheff".to_string(),
            time: 60,
            revenue: 200_000,
            cost: 70_000,
        },
        Band {
            name: "Caffeine Buzz".to_string(),
            time: 70,
            revenue: 100_000,
            cost: 45_000,
        },
        Band {
            name: "Caffeine Buzz".to_string(),
            time: 70,
            revenue: 100_000,
            cost: 45_000,
        },
        Band {
            name: "647 Buckingham Way".to_string(),
            time: 100,
            revenue: 600_000,
            cost: 80_000,
        },
        Band {
            name: "Hummingbird Anthem".to_string(),
            time: 60,
            revenue: 400_000,
            cost: 60_000,
        },
        Band {
            name: "Macaulay & Co.".to_string(),
            time: 80,
            revenue: 300_000,
            cost: 220_000,
        },
        Band {
            name: "Sonderbund".to_string(),
            time: 70,
            revenue: 600_000,
            cost: 120_000,
        },
        Band {
            name: "Onyx Eyes".to_string(),
            time: 90,
            revenue: 800_000,
            cost: 190_000,
        },
        Band {
            name: "DJ Megara".to_string(),
            time: 60,
            revenue: 250_000,
            cost: 50_000,
        },
        Band {
            name: "Sloth Central Incorporated".to_string(),
            time: 50,
            revenue: 150_000,
            cost: 45_000,
        },
        Band {
            name: "Sloth Central Incorporated".to_string(),
            time: 50,
            revenue: 150_000,
            cost: 45_000,
        },
        Band {
            name: "Fractured Coccyx".to_string(),
            time: 90,
            revenue: 400_000,
            cost: 200_000,
        },
        Band {
            name: "Forgotten Indigo".to_string(),
            time: 30,
            revenue: 50_000,
            cost: 0,
        },
    ];
    let mut best_econochella = Econochella::new(1_370_000, 0.5, knapsack);
    let mut running_econochella = best_econochella.clone();

    let mut rng = rand::thread_rng();

    for _ in 0..1_000 {
        let mut temp_econochella = best_econochella.clone();
        let (band, current_location) = temp_econochella.choose_band(&mut rng);
        // choose a random location. choosing the same location amounts to a deletion and a random reinsertion
        let new_location = current_location.choose_location(&mut rng);
        // move the band to a random time in the new location's schedule
        temp_econochella.move_band(band, current_location, new_location, &mut rng);
        if temp_econochella.valid() {
            let delta = temp_econochella.value() as f64 - running_econochella.value() as f64;
            if delta > 0.0 || rng.gen::<f64>() < (-delta / TEMPERATURE).exp() {
                running_econochella = temp_econochella;
            }
            if running_econochella.value() > best_econochella.value() {
                best_econochella = running_econochella.clone();
            }
        }
        print!("{}\t", running_econochella.value());
    }
    println!("\n The cost is {}, the times are tent: {}, amphitheater: {}, stadium: {}", best_econochella.tent.cost() +best_econochella.amphitheater.cost() + best_econochella.stadium.cost(), best_econochella.tent.time(), best_econochella.amphitheater.time(), best_econochella.stadium.time());
    println!("Value: {}, \ntent schedule: {:?}, \namphitheater schedule: {:?}, \nstadium schedule: {:?}", best_econochella.value(), best_econochella.tent.schedule, best_econochella.amphitheater.schedule, best_econochella.stadium.schedule);

}

