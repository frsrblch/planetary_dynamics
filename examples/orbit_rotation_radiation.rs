use fractional_int::FractionalU8;
use orbital_mechanics::pga::{line, motor, origin, point, Bivector, Dot, RightComp, Sandwich};
use orbital_mechanics::{Eccentricity, EllipticalOrbit, Rotation};
use physics_types::{
    Angle, Area, Duration, Energy, EnergyPerTemperature, FluxDensity, Length, Power, Temperature,
    TimeFloat, AU, J, K, KM, YR,
};
use planetary_dynamics::adjacency::{rotations, AdjArray, Adjacency, Node};
use planetary_dynamics::solar_radiation::{Albedo, InfraredTransparency, RadiativeAbsorption};
use planetary_dynamics::terrain::Terrain;
use planetary_dynamics::tile_gen::create_terrain;
use plotters::prelude::*;

// TODO decouple system.dt and heat transfer
// TODO heat capacity based on terrain (water's is higher and it has mixing)
// TODO heat transfer based on terrain and neighbours
// TODO add atmospheres (affects: clouds, albedo, and infrared reflectance)
// TODO elevation effects on temperature (9.8 K / km)
// consider what elevation would allow ice to accumulate for adding glaciers

const N: usize = 24;
const DT: Duration = Duration::in_hr(0.2);

pub fn main() {
    let mut system = System::earth();

    system.get_min_max_step(system.duration, DT);

    let start = std::time::Instant::now();
    let temps = system.get_min_max(system.duration, Duration::in_d(1.0), DT);
    let end = std::time::Instant::now();
    let elapsed = end - start;
    println!("{} ms", elapsed.as_millis());

    let steps = temps.len();

    let min = temps
        .iter()
        .flat_map(|v| v.iter())
        .map(|t| t.0)
        .min()
        .unwrap()
        .value
        - 273.15;

    let max = temps
        .iter()
        .flat_map(|v| v.iter())
        .map(|t| t.1)
        .max()
        .unwrap()
        .value
        - 273.15;

    let avg = {
        let count = temps.iter().flat_map(|v| v.iter()).count() * 2;
        let sum = temps
            .iter()
            .flat_map(|v| v.iter().map(|t| t.0 + t.1))
            .sum::<Temperature>();
        (sum / count as f64).value - 273.15
    };
    println!("avg: {:.1} C ({:.1} - {:.1})", avg, min, max);

    let area = SVGBackend::new("plot.svg", (1024, 760)).into_drawing_area();
    area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&area)
        .build_cartesian_3d(0f64..1f64, min.floor()..max.ceil(), 0f64..1f64)
        .unwrap();

    chart.with_projection(|mut pb| {
        pb.yaw = 2.5;
        pb.scale = 0.9;
        pb.into_matrix()
    });

    chart.configure_axes().draw().unwrap();

    for tile in 0..N {
        let t = tile as f64 / N as f64;
        let iter = temps.iter().enumerate().map(|(i, v)| {
            let s = i as f64 / steps as f64;
            let (min, _) = v[tile];

            (t, min.value - 273.15, s)
        });
        chart.draw_series(LineSeries::new(iter, &BLUE)).unwrap();

        let iter = temps.iter().enumerate().map(|(i, v)| {
            let s = i as f64 / steps as f64;
            let (_, max) = v[tile];
            (t, max.value - 273.15, s)
        });
        chart.draw_series(LineSeries::new(iter, &RED)).unwrap();
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()
        .unwrap();

    // std::fs::write("sim.txt", &output).ok();
}

struct System {
    star: Power,
    duration: Duration,
    orbit: EllipticalOrbit,
    axis: Rotation,
    surfaces: Vec<Bivector>,
    adj: Vec<AdjArray>,
    temp: Vec<Temperature>,
    neighbour_avg_temp: Vec<Temperature>,
    heat_trapping: InfraredTransparency,
    emissivity: f64,
    heat_capacity: EnergyPerTemperature,
    time: TimeFloat,
    dt: Duration,
    terrain: Vec<Terrain>,
    clouds: FractionalU8,
    heat_transfer: f64,
    radiative_absorption: RadiativeAbsorption,
}

fn sun() -> Power {
    Power::blackbody(5772.0 * K, 695_700.0 * KM)
}

impl System {
    pub fn earth() -> Self {
        let mut adj = Adjacency::default();
        adj.register(N);

        let mut terrain = create_terrain(N, 0.7, &adj);
        terrain[0] = Terrain::new_fraction(1.0, 0.0, 1.0);
        terrain[1].glacier = FractionalU8::new_f64(0.75);
        terrain[2].glacier = FractionalU8::new_f64(0.5);
        terrain[3].glacier = FractionalU8::new_f64(0.25);
        terrain[N - 1] = Terrain::new_fraction(0.0, 0.5, 1.0);
        terrain[N - 2].glacier = FractionalU8::new_f64(0.75);
        terrain[N - 3].glacier = FractionalU8::new_f64(0.5);
        terrain[N - 4].glacier = FractionalU8::new_f64(0.25);

        let adj = adj.get(N).clone();

        let angle = Angle::in_deg(23.439);
        let axial_tilt = motor(line(origin(), point(0.0, 1.0, 0.0)), 0.0, angle.value);

        System {
            star: sun(),
            duration: YR,
            orbit: EllipticalOrbit {
                period: YR,
                semi_major_axis: AU,
                eccentricity: Eccentricity::new(0.0167),
                eccentricity_angle: Default::default(),
                offset: Default::default(),
            },
            axis: Rotation {
                sidereal_speed: Angle::TAU / Duration::in_d(0.99726968),
                axis: {
                    let (sin, cos) = Angle::in_deg(23.439).sin_cos();
                    line(origin(), point(sin, 0.0, cos))
                },
            },
            surfaces: (0..N as u16)
                .into_iter()
                .map(|n| Node::new(n, N as u16).position(rotations(N as u16)))
                .map(|p| line(origin(), point(p.x, p.y, p.z)).r_comp())
                .map(|p| axial_tilt.sandwich(p))
                .collect::<Vec<_>>(),
            adj,
            temp: vec![Temperature::in_c(15.0); N],
            neighbour_avg_temp: vec![Temperature::default(); N],
            heat_trapping: InfraredTransparency::new(0.5),
            emissivity: 0.93643,
            heat_capacity: 1.5e6 * J / K,
            time: Default::default(),
            dt: Duration::in_hr(0.2),
            terrain,
            clouds: FractionalU8::new_f64(0.52),
            heat_transfer: 0.995,
            radiative_absorption: !Albedo::new(0.18),
        }
    }

    pub fn mars() -> Self {
        let mut adj = Adjacency::default();
        adj.register(N);

        let terrain = create_terrain(N, 0.0, &adj);
        let adj = adj.get(N).clone();

        let angle = Angle::in_deg(25.19);
        let axial_tilt = motor(line(origin(), point(0.0, 1.0, 0.0)), 0.0, angle.value);

        System {
            star: sun(),
            duration: Duration::in_d(686.980),
            orbit: EllipticalOrbit {
                period: Duration::in_d(686.980),
                semi_major_axis: Length::in_m(227_939_200e3),
                eccentricity: Eccentricity::new(0.0934),
                eccentricity_angle: Default::default(),
                offset: Default::default(),
            },
            axis: Rotation {
                sidereal_speed: Angle::TAU / Duration::in_d(1.025957),
                axis: {
                    let (sin, cos) = angle.sin_cos();
                    line(origin(), point(sin, 0.0, cos))
                },
            },
            surfaces: (0..N as u16)
                .into_iter()
                .map(|n| Node::new(n, N as u16).position(rotations(N as u16)))
                .map(|p| line(origin(), point(p.x, p.y, p.z)).r_comp())
                .map(|s| axial_tilt.sandwich(s))
                .collect::<Vec<_>>(),
            adj,
            temp: vec![Temperature::in_k(210.0); N],
            neighbour_avg_temp: vec![Temperature::default(); N],
            heat_trapping: InfraredTransparency::new(0.91),
            emissivity: 0.9,
            heat_capacity: Energy::in_joules(1e5) / Temperature::in_k(1.0),
            time: Default::default(),
            dt: Duration::in_hr(0.5),
            terrain,
            clouds: FractionalU8::default(),
            heat_transfer: 0.99,
            radiative_absorption: !Albedo::new(0.25),
        }
    }

    fn get_min_max(
        &mut self,
        duration: Duration,
        step: Duration,
        dt: Duration,
    ) -> Vec<Vec<(Temperature, Temperature)>> {
        assert!(duration > step);

        let mut output = vec![];
        let target = self.time + duration;

        while self.time < target {
            let min_max = self.get_min_max_step(step, dt);
            output.push(min_max);
        }

        output
    }

    fn get_min_max_step(
        &mut self,
        step: Duration,
        dt: Duration,
    ) -> Vec<(Temperature, Temperature)> {
        assert!(step > self.dt);

        let target = self.time + step;

        self.advance(dt);

        let mut min_max = self.temp.iter().map(|t| (*t, *t)).collect::<Vec<_>>();

        while self.time < target {
            self.advance(dt);
            for ((min, max), temp) in min_max.iter_mut().zip(self.temp.iter()) {
                *min = (*min).min(*temp);
                *max = (*max).max(*temp);
            }
        }

        min_max
    }

    fn advance(&mut self, dt: Duration) {
        let pos = self.orbit.distance(self.time);
        let ray = line(origin(), point(pos.x.value, pos.y.value, 0.0)).r_comp();
        let flux_density = self.star / pos.magnitude_squared();

        let motor = self.axis.get_motor(self.time);

        let iter = self
            .temp
            .iter_mut()
            .zip(self.surfaces.iter())
            .zip(self.terrain.iter());

        for ((temp, surface), terrain) in iter {
            let surface = motor.sandwich(*surface);
            let intensity = (-surface.dot(ray)).max(0.0);

            let ra = terrain.absorption(self.radiative_absorption, self.clouds);

            let flux_density = flux_density * intensity * ra.0.powf((1.0 / intensity).powf(0.678));
            // let flux_density = flux_density * intensity * ra;

            let emission = FluxDensity::blackbody(*temp) * self.heat_trapping * self.emissivity;

            let d_energy = (flux_density - emission) * Area::in_m2(1.0) * dt;
            let d_temp = d_energy / self.heat_capacity;
            *temp += d_temp;
        }

        let temp = &mut self.temp;
        for (i, neighbour_avg_temp) in self.neighbour_avg_temp.iter_mut().enumerate() {
            let mut count = 0;
            let mut sum = Temperature::default();
            self.adj[i].iter().for_each(|n| {
                count += 1;
                sum += temp[n];
            });
            *neighbour_avg_temp = sum / count as f64;
        }

        let heat_transfer = 1.0 - self.heat_transfer.powf(dt.value / 3600.0);
        for (temp, avg_temp) in temp.iter_mut().zip(self.neighbour_avg_temp.iter()) {
            *temp += (*avg_temp - *temp) * heat_transfer;
        }

        self.time += dt;
    }
}
