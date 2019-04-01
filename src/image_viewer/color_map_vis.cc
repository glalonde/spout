#include <thread>
#include "base/init.h"
#include "base/time.h"
#include "base/wall_timer.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/image_viewer/image_viewer.h"

DEFINE_int32(width, 256, "display width");
DEFINE_int32(height, 256, "display height");

DEFINE_double(texture_scale, 1.0, "texture scale factor");
DEFINE_int32(color_map_index, 0, "Color map index, see color_maps.h");

// Set the data in the image to a radial gradient
void SetToGradient(const ColorMap map, double phase,
                   Image<PixelType::RGBAU8>* data) {
  // (row, col)
  const Vector2d center = Vector2d(data->rows(), data->cols()) / 2.0;
  const double radius = center.norm();
  // (row, col)
  Vector2d current;
  for (int c = 0; c < data->cols(); ++c) {
    current[1] = c;
    for (int r = 0; r < data->rows(); ++r) {
      current[0] = r;
      const double normalized_distance = (center - current).norm() / radius;
      const double map_index = std::fmod(normalized_distance + phase, 1.0);
      const Vector3f color = GetMappedColor3f(map, map_index);
      (*data)(r, c) = Convert<PixelType::RGBAU8, PixelType::RGBF32>(color);
    }
  }
}

class CircularBuffer {
 public:
  CircularBuffer(int capacity) : write_index_(0) {
    data_.reserve(capacity);
  }

  std::optional<Duration> Push(Duration value) {
    if (data_.size() < data_.capacity()) {
      data_.push_back(value);
      return {};
    } else {
      Duration removed = data_[write_index_];
      data_[write_index_] = value;
      ++write_index_;
      if (write_index_ >= data_.capacity()) {
        write_index_ = 0;
      }
      return removed;
    }
  }

  int WriteIndex() {
    return write_index_;
  }

  const std::vector<Duration>& data() {
    return data_;
  }

 private:
  int write_index_;
  std::vector<Duration> data_;
};

class FPSEstimator {
 public:
  FPSEstimator(Duration window, double estimated_max_frequency)
      : deltas_(InitBuffer(window, estimated_max_frequency)), sum_(0) {}

  double Tick(Duration delta) {
    sum_ += delta;
    auto maybe_removed = deltas_.Push(delta);
    if (maybe_removed) {
      sum_ -= *maybe_removed;
    }
    return (1 + deltas_.data().size()) / ToSeconds<double>(sum_);
  }

 private:
  static CircularBuffer InitBuffer(Duration window,
                                   double estimated_max_frequency) {
    const int estimated_number = static_cast<int>(
        std::ceil(ToSeconds<double>(window) * estimated_max_frequency));
    return CircularBuffer(estimated_number);
  }

  CircularBuffer deltas_;
  Duration sum_;
};

int main(int argc, char* argv[]) {
  Init(argc, argv);
  ImageViewer viewer(FLAGS_width, FLAGS_height);
  int tex_width = FLAGS_width * FLAGS_texture_scale;
  int tex_height = FLAGS_height * FLAGS_texture_scale;
  viewer.SetTextureSize(tex_width, tex_height);

  auto* data = viewer.data();
  CHECK_GE(FLAGS_color_map_index, 0);
  CHECK_LT(FLAGS_color_map_index, kAllColorMaps.size());
  SetToGradient(kAllColorMaps[FLAGS_color_map_index], 0, data);
  constexpr double kTargetFPS = 60.0;
  constexpr Duration kTargetCycleTime = FromHz(kTargetFPS);

  FPSEstimator estimator(FromSeconds(1.0), kTargetFPS);
  WallTimer timer;
  timer.Start();
  viewer.ToggleFullScreen();

  bool done = false;
  TimePoint previous = ClockType::now();
  while (!done) {
    auto current = ClockType::now();
    auto target_finish = current + kTargetCycleTime;
    estimator.Tick(current - previous);
    previous = current;
    SetToGradient(kAllColorMaps[FLAGS_color_map_index],
                  ToSeconds<double>(timer.ElapsedDuration()), data);
    viewer.SetDataChanged();
    done = viewer.Update().quit;
    std::this_thread::sleep_until(target_finish);
  }
  return 0;
}
