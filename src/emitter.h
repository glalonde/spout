#pragma once
#include <random>
#include "base/circular_buffer.h"
#include "base/logging.h"
#include "src/eigen_types.h"
#include "src/so2.h"

class Emitter {
 public:
  Emitter(double angular_stdev, double min_speed, double max_speed,
          double emission_rate, double particle_life, int random_seed = 0)
      : angular_dist_(0, angular_stdev),
        speed_dist_(min_speed, max_speed),
        emission_period_(1.0 / emission_rate),
        particle_life_(particle_life),
        rand_gen_(random_seed),
        particles_(static_cast<int>(std::ceil(emission_rate * particle_life)),
                   Vector5d()),
        emission_progress_(0) {
    CHECK_LT(min_speed, max_speed);
  }

  // Compute the number of emissions and then distribute them linearly over the
  // range of configurations
  void EmitOverTime(const double dt, const SO2d& start_angle,
                    const SO2d& end_angle, const Vector2d& start_pos,
                    const Vector2d& end_pos, const Vector2d& start_vel,
                    const Vector2d& end_vel) {
    emission_progress_ += dt;
    if (emission_progress_ > emission_period_) {
      const int num_emissions =
          static_cast<int>(emission_progress_ / emission_period_);
      emission_progress_ -= num_emissions * emission_period_;
      EmitOverSpace(num_emissions, start_angle, end_angle, start_pos, end_pos,
                    start_vel, end_vel);
    }
  }

  // Emit N particles over a linear interpolation between start and end states.
  void EmitOverSpace(const int num_emissions, const SO2d& start_angle,
                     const SO2d& end_angle, const Vector2d& start_pos,
                     const Vector2d& end_pos, const Vector2d& start_vel,
                     const Vector2d& end_vel) {
    if (num_emissions <= 0) {
      return;
    }
    const double angle_diff = (start_angle.inverse() * end_angle).radians();
    const SO2d angle_step = SO2d(angle_diff / (num_emissions + 1));
    const Vector2d pos_step = (end_pos - start_pos) / (num_emissions + 1);
    const Vector2d vel_step = (end_vel - start_vel) / (num_emissions + 1);
    Vector2d pos = start_pos;
    Vector2d vel = start_vel;
    SO2d angle = start_angle;
    EmitOne(pos, vel, angle);
    for (int i = 1; i < num_emissions; ++i) {
      pos += pos_step;
      vel += vel_step;
      angle *= angle_step;
      EmitOne(pos, vel, angle);
    }
  }

  // Emit one particle drawing around the specified configuration, applying
  // noise from the random distributions
  void EmitOne(const Vector2d& pos, const Vector2d& velocity_offset,
               const SO2d& angle) {
    Vector5d out;
    out.head<2>() = pos;
    const SO2d emission_angle = angle * SO2d(angular_dist_(rand_gen_));
    out.segment<2>(2) =
        emission_angle.data() * speed_dist_(rand_gen_) + velocity_offset;
    out[4] = particle_life_;
    particles_.Push(out);
  }

  const CircularBuffer<Vector5d>& particles() const {
    return particles_;
  }
  CircularBuffer<Vector5d>* mutable_particles() {
    return &particles_;
  }

 private:
  // Emitter properties
  std::normal_distribution<double> angular_dist_;
  std::uniform_real_distribution<double> speed_dist_;
  double emission_period_;
  double particle_life_;

  // State
  std::mt19937 rand_gen_;
  CircularBuffer<Vector5d> particles_;
  // RingBuffer<Particle> particles;
  // Keep track of partial emission progress across timesteps with this
  double emission_progress_;
};
