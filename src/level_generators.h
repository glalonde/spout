#pragma once
#include "src/image.h"
#include "src/random.h"

template <class T>
void GenerateRectangleLevel(int max_dimension, int num_vacancies,
                            const T& min_obs_val, const T& max_obs_val,
                            uint32_t level_seed, Image<T>* data) {
  max_dimension =
      std::max(1, std::min(static_cast<int>(data->cols()), max_dimension));
  std::mt19937 gen(level_seed);
  Image<double> perlin_vals(data->rows(), data->cols());
  PerlinNoise(40, &gen, perlin_vals);
  perlin_vals *= .5;
  perlin_vals += .5;
  *data = (perlin_vals * 255).cast<uint8_t>();

  std::uniform_int_distribution<int> dim_dist(1, max_dimension);
  const int level_height = data->rows();
  const int level_width = data->cols();
  for (int i = 0; i < num_vacancies; i++) {
    const int width = dim_dist(gen);
    const int height = dim_dist(gen);
    std::uniform_int_distribution<int> left_dist(0, level_width - width);
    std::uniform_int_distribution<int> bot_dist(0, level_height - height);
    const int left = left_dist(gen);
    const int bot = bot_dist(gen);
    data->block(bot, left, height, width).setConstant(T(0));
  }
}

template <class T>
void MakeLevel(const T& min_obs_val, const T& max_obs_val, int level_num,
               uint32_t level_seed, Image<T>* data) {
  const int max_dimension = static_cast<int>(data->cols() / level_num) / 2;
  const int num_vacancies =
      static_cast<int>(data->rows() * std::sqrt(level_num));
  GenerateRectangleLevel(max_dimension, num_vacancies, min_obs_val, max_obs_val,
                         level_seed, data);
  if (level_num <= 1) {
    constexpr double kFirstLevelEmptyPortion = .5;
    const int first_level_empty_rows =
        static_cast<int>(data->rows() * kFirstLevelEmptyPortion);
    data->block(0, 0, first_level_empty_rows, data->cols()).setConstant(T(0));
  }
}
