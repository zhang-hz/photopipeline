#include "Halide.h"

namespace {

enum ColorSpace : int { LinearSRGB = 0, Rec2020PQ = 1 };

ColorSpace parse_color_space(const std::string &s) {
    if (s == "linear_srgb") return LinearSRGB;
    if (s == "rec2020_pq") return Rec2020PQ;
    return LinearSRGB;
}

float srgb_to_linear(float x) {
    if (x <= 0.04045f) return x / 12.92f;
    return Halide::pow((x + 0.055f) / 1.055f, 2.4f);
}

float linear_to_srgb(float x) {
    if (x <= 0.0031308f) return 12.92f * x;
    return 1.055f * Halide::pow(x, 1.0f / 2.4f) - 0.055f;
}

float linear_to_pq(float x) {
    const float m1 = 0.1593017578125f;
    const float m2 = 78.84375f;
    const float c1 = 0.8359375f;
    const float c2 = 18.8515625f;
    const float c3 = 18.6875f;
    float x_pow = Halide::pow(x, m1);
    float num = c1 + c2 * x_pow;
    float den = 1.0f + c3 * x_pow;
    float y = Halide::pow(num / den, m2);
    return y;
}

float pq_to_linear(float x) {
    const float m1 = 0.1593017578125f;
    const float m2 = 78.84375f;
    const float c1 = 0.8359375f;
    const float c2 = 18.8515625f;
    const float c3 = 18.6875f;
    float y = Halide::pow(x, 1.0f / m2);
    float num = Halide::max(y - c1, 0.0f) / (c2 - c3 * y);
    return Halide::pow(num, 1.0f / m1);
}

Halide::Expr apply_transfer_linear_to_srgb(Halide::Expr x) {
    return Halide::select(x <= 0.0031308f, 12.92f * x,
                          1.055f * Halide::pow(x, 1.0f / 2.4f) - 0.055f);
}

Halide::Expr apply_transfer_srgb_to_linear(Halide::Expr x) {
    return Halide::select(x <= 0.04045f, x / 12.92f,
                          Halide::pow((x + 0.055f) / 1.055f, 2.4f));
}

Halide::Expr apply_transfer_linear_to_pq(Halide::Expr x) {
    Halide::Expr m1 = 0.1593017578125f;
    Halide::Expr m2 = 78.84375f;
    Halide::Expr c1 = 0.8359375f;
    Halide::Expr c2 = 18.8515625f;
    Halide::Expr c3 = 18.6875f;
    Halide::Expr x_pow = Halide::pow(Halide::max(x, 0.0f), m1);
    Halide::Expr num = c1 + c2 * x_pow;
    Halide::Expr den = 1.0f + c3 * x_pow;
    return Halide::pow(num / den, m2);
}

Halide::Expr apply_transfer_pq_to_linear(Halide::Expr x) {
    Halide::Expr m1 = 0.1593017578125f;
    Halide::Expr m2 = 78.84375f;
    Halide::Expr c1 = 0.8359375f;
    Halide::Expr c2 = 18.8515625f;
    Halide::Expr c3 = 18.6875f;
    Halide::Expr y = Halide::pow(Halide::max(x, 0.0f), 1.0f / m2);
    Halide::Expr num = Halide::max(y - c1, 0.0f) / (c2 - c3 * y);
    return Halide::pow(num, 1.0f / m1);
}

Halide::Expr decode_transfer(Halide::Expr x, ColorSpace cs) {
    switch (cs) {
        case LinearSRGB: return x;
        case Rec2020PQ:  return apply_transfer_pq_to_linear(x);
    }
    return x;
}

Halide::Expr encode_transfer(Halide::Expr x, ColorSpace cs) {
    switch (cs) {
        case LinearSRGB: return x;
        case Rec2020PQ:  return apply_transfer_linear_to_pq(x);
    }
    return x;
}

void convert_primaries(Halide::Expr &r, Halide::Expr &g, Halide::Expr &b,
                       ColorSpace src, ColorSpace dst) {
    if (src == dst) return;
    if (src == LinearSRGB && dst == Rec2020PQ) {
        Halide::Expr r2 = 0.6274f * r + 0.3293f * g + 0.0433f * b;
        Halide::Expr g2 = 0.0691f * r + 0.9195f * g + 0.0114f * b;
        Halide::Expr b2 = 0.0164f * r + 0.0880f * g + 0.8956f * b;
        r = r2; g = g2; b = b2;
    } else if (src == Rec2020PQ && dst == LinearSRGB) {
        Halide::Expr r2 =  1.6605f * r - 0.5876f * g - 0.0728f * b;
        Halide::Expr g2 = -0.1246f * r + 1.1329f * g - 0.0083f * b;
        Halide::Expr b2 = -0.0182f * r - 0.1006f * g + 1.1187f * b;
        r = r2; g = g2; b = b2;
    }
}

} // namespace

class ColorSpaceConvert : public Halide::Generator<ColorSpaceConvert> {
public:
    GeneratorParam<std::string> source{"source", "linear_srgb"};
    GeneratorParam<std::string> target{"target", "rec2020_pq"};

    Input<Halide::Buffer<float, 3>> input{"input"};
    Output<Halide::Buffer<float, 3>> output{"output"};

    Var x{"x"}, y{"y"}, c{"c"};

    void generate() {
        ColorSpace src = parse_color_space(source);
        ColorSpace dst = parse_color_space(target);

        Halide::Expr val = input(x, y, c);

        Halide::Expr r = input(x, y, 0);
        Halide::Expr g = input(x, y, 1);
        Halide::Expr b = input(x, y, 2);

        r = decode_transfer(r, src);
        g = decode_transfer(g, src);
        b = decode_transfer(b, src);

        convert_primaries(r, g, b, src, dst);

        r = encode_transfer(r, dst);
        g = encode_transfer(g, dst);
        b = encode_transfer(b, dst);

        output(x, y, c) = Halide::mux(c, {r, g, b});
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

HALIDE_REGISTER_GENERATOR(ColorSpaceConvert, colorspace_convert)
