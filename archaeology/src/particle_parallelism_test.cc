#include "base/init.h"
#include "base/logging.h"
#include "base/wall_timer.h"
#include "src/eigen_types.h"

// X, Y, DX, DY, TTL
template <class Scalar>
using Particle = Vector<Scalar, 5>;

// Delta particle (delta dx, delta dy, delta TTL)
template <class Scalar>
using DParticle = Vector<Scalar, 3>;

// Update a single particle
template <class Scalar>
inline void UpdateParticle(const DParticle<Scalar>& dp,
                           Particle<Scalar>* particle_out) {
  // Apply velocity
  particle_out->template head<2>() +=
      (-dp[2] /* delta ttl is -dt */) * particle_out->template segment<2>(2);
  // Apply acceleration
  particle_out->template tail<3>() += dp;
}

template <class Scalar>
Eigen::AlignedBox<Scalar, 5> MakeParticleSpace(Scalar space, Scalar velocity,
                                               Scalar time) {
  Eigen::AlignedBox<Scalar, 5> particle_space;
  Particle<Scalar> max_particle;
  Particle<Scalar> min_particle;
  max_particle << space, space, velocity, velocity, time;
  min_particle << -space, -space, -velocity, -velocity, 0;
  particle_space.extend(max_particle);
  particle_space.extend(min_particle);
  return particle_space;
}


void Test1(int num_particles, int num_iterations) {
  const auto particle_space = MakeParticleSpace<float>(10, 2, 10);
  std::vector<Particle<float>> normal_particles(num_particles);
  for (int i = 0; i < num_particles; ++i) {
    normal_particles[i] = particle_space.sample();
  }

  const float dt = 1.0 / 60;
  const float ddy = -9.81 * dt;
  const float ddx = 0 * dt;
  Vector3<float> dp(ddx, ddy, -dt);

  WallTimer timer;
  timer.Start();
  for (int i = 0; i < num_iterations; ++i) {
    for (int j = 0; j < num_particles; ++j) {
      UpdateParticle(dp, &normal_particles[j]);
    }
  }
  timer.Stop();
  LOG(INFO) << timer.ElapsedDuration();
  LOG(INFO) << num_particles / ToSeconds<double>(timer.ElapsedDuration())
            << " particles per second";
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Test1(std::pow(2, 20), 100);
  return 0;
}
