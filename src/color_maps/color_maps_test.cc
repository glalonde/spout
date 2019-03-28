#include "src/color_maps/color_maps.h"
#include "base/googletest.h"

GTEST_TEST(ColorMaps, AllRange) {
  auto is_valid_color_channel = [](auto val) {
    return !std::isnan(val) && val >= 0 && val <= 1;
  };
  const VectorXd vals = VectorXd::LinSpaced(100, -.1, 1.1);
  for (const auto& map : kAllColorMaps) {
    for (int i = 0; i < vals.size(); ++i) {
      Vector3f color = GetMappedColor3f(map, vals[i]);
      for (int j = 0; j < 3; ++j) {
        EXPECT_TRUE(is_valid_color_channel(color[j])) << vals[i];
      }
    }
  }
}

GTEST_MAIN();
