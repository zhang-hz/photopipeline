#include "Halide.h"

namespace {

float lanczos3(float x) {
    if (x == 0.0f) return 1.0f;
    if (x < -3.0f || x > 3.0f) return 0.0f;
    float a = 3.0f;
    float ax = std::abs(x);
    float pix = Halide::Internal::pi * x;
    float pia = Halide::Internal::pi * (x / a);
    return a * std::sin(pix) * std::sin(pia) / (pix * pix);
}

float triangle(float x) {
    float ax = std::abs(x);
    return ax < 1.0f ? 1.0f - ax : 0.0f;
}

Halide::Func make_lanczos3_kernel() {
    Halide::Func k{"lanczos3"};
    Halide::Var x{"xk"};
    const int radius = 3;
    const int taps = 2 * radius + 1;

    Halide::Func s;
    s(x) = 0.0f;
    for (int i = -radius; i <= radius; ++i) {
        float w = lanczos3(static_cast<float>(i));
        s(x + i + radius) += w;
    }

    k(x) = 0.0f;
    for (int i = -radius; i <= radius; ++i) {
        float w = lanczos3(static_cast<float>(i));
        k(x) += Halide::select(
            x == i + radius,
            Halide::cast<float>(w) / s(i + radius),
            0.0f);
    }

    return k;
}

} // namespace

class Resize : public Halide::Generator<Resize> {
public:
    GeneratorParam<std::string> filter_type{"filter_type", "lanczos3"};

    Input<Halide::Buffer<float, 3>> input{"input"};
    Input<int> target_width{"target_width"};
    Input<int> target_height{"target_height"};

    Output<Halide::Buffer<float, 3>> output{"output"};

    void generate() {
        Halide::Expr in_w = input.dim(0).extent();
        Halide::Expr in_h = input.dim(1).extent();
        Halide::Expr channels = input.dim(2).extent();

        Halide::Expr scale_x = Halide::cast<float>(in_w) / Halide::cast<float>(target_width);
        Halide::Expr scale_y = Halide::cast<float>(in_h) / Halide::cast<float>(target_height);

        Halide::Var x{"x"}, y{"y"}, c{"c"};

        Halide::Expr fx = Halide::cast<float>(x) * scale_x + 0.5f;

        Halide::Func lanczos3_kernel = make_lanczos3_kernel();
        const int radius = 3;
        const int taps = 2 * radius + 1;

        Halide::Expr clamped_x = Halide::clamp(Halide::cast<int>(Halide::floor(fx)), 0, in_w - 1);

        Halide::Func horizontal{"horizontal"};
        Halide::Var xi{"xi"};
        horizontal(x, y, c) = 0.0f;
        for (int i = -radius; i <= radius; ++i) {
            Halide::Expr sx = Halide::clamp(clamped_x + i, 0, in_w - 1);
            float w = lanczos3(static_cast<float>(i));
            horizontal(x, y, c) += input(sx, y, c) * w;
        }

        Halide::Expr fy = Halide::cast<float>(y) * scale_y + 0.5f;
        Halide::Expr clamped_y = Halide::clamp(Halide::cast<int>(Halide::floor(fy)), 0, in_h - 1);

        output(x, y, c) = 0.0f;
        for (int j = -radius; j <= radius; ++j) {
            Halide::Expr sy = Halide::clamp(clamped_y + j, 0, in_h - 1);
            float w = lanczos3(static_cast<float>(j));
            output(x, y, c) += horizontal(x, sy, c) * w;
        }
    }

    void schedule() {
        output.dim(0).set_stride(1);
        output.dim(2).set_stride(output.dim(0).extent());

        if (output.dim(1).extent() >= 16) {
            Var yo{"yo"}, yi{"yi"};
            output.split(y, yo, yi, 16).parallel(yo).vectorize(x, 8);
        } else {
            output.vectorize(x, 8);
        }
    }
};

HALIDE_REGISTER_GENERATOR(Resize, resize)
