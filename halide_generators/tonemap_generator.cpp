#include "Halide.h"

namespace {

enum TonemapAlgorithm : int { Reinhard = 0, ACES = 1 };

TonemapAlgorithm parse_algorithm(const std::string &s) {
    if (s == "reinhard") return Reinhard;
    if (s == "aces") return ACES;
    return Reinhard;
}

Halide::Expr luminance(Halide::Expr r, Halide::Expr g, Halide::Expr b) {
    return 0.2126f * r + 0.7152f * g + 0.0722f * b;
}

Halide::Expr reinhard_tone(Halide::Expr channel, Halide::Expr lum) {
    return channel / (channel + 1.0f);
}

Halide::Expr aces_fit(Halide::Expr x) {
    const float A = 2.51f;
    const float B = 0.03f;
    const float C = 2.43f;
    const float D = 0.59f;
    const float E = 0.14f;
    return Halide::clamp((x * (A * x + B)) / (x * (C * x + D) + E), 0.0f, 1.0f);
}

} // namespace

class Tonemap : public Halide::Generator<Tonemap> {
public:
    GeneratorParam<std::string> algorithm{"algorithm", "reinhard"};
    GeneratorParam<float> exposure{"exposure", 1.0f};

    Input<Halide::Buffer<float, 3>> input{"input"};
    Input<float> max_luminance{"max_luminance"};

    Output<Halide::Buffer<float, 3>> output{"output"};

    Var x{"x"}, y{"y"}, c{"c"};

    void generate() {
        TonemapAlgorithm algo = parse_algorithm(algorithm);

        Halide::Expr r = input(x, y, 0) * exposure;
        Halide::Expr g = input(x, y, 1) * exposure;
        Halide::Expr b = input(x, y, 2) * exposure;

        Halide::Expr lum = luminance(r, g, b);

        Halide::Expr scale;
        if (algo == Reinhard) {
            scale = 1.0f / (1.0f + lum / Halide::max(max_luminance, 0.001f));
        } else {
            scale = 1.0f;
        }

        Halide::Expr r_toned, g_toned, b_toned;

        if (algo == Reinhard) {
            r_toned = Halide::clamp(r * scale, 0.0f, 1.0f);
            g_toned = Halide::clamp(g * scale, 0.0f, 1.0f);
            b_toned = Halide::clamp(b * scale, 0.0f, 1.0f);
        } else {
            r_toned = aces_fit(r);
            g_toned = aces_fit(g);
            b_toned = aces_fit(b);
        }

        output(x, y, c) = Halide::mux(c, {r_toned, g_toned, b_toned});
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

HALIDE_REGISTER_GENERATOR(Tonemap, tonemap)
