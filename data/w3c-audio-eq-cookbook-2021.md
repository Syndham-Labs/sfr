# Cookbook formulae for audio equalizer biquad filter coefficients

Author [Robert Bristow-Johnson](mailto:rbj@audioimagination.com) Editors [Raymond Toy](mailto:toy.raymond@gmail.com) [Doug Schepers (previously W3C)](mailto:schepers@w3.org)

Adapted from [Audio-EQ-Cookbook.txt](https://webaudio.github.io/Audio-EQ-Cookbook/Audio-EQ-Cookbook.txt), by Robert Bristow-Johnson, with permission.

## Biquad Filter Formulae

All filter transfer functions were derived from analog prototypes (that are shown below for each equalizer (EQ) filter type) and had been digitized using the Bilinear Transform (BLT). BLT frequency warping has been taken into account for both significant frequency relocation (this is the normal "prewarping" that is necessary when using the BLT) and for bandwidth readjustment (since the bandwidth is compressed when mapped from analog to digital using the BLT).

First, given a biquad transfer function defined as:

```
      $$
        \begin{equation}
          H(z) = \frac{b_0 + b_1z^{-1}+b_2z^{-2}}{a_0 + a_1z^{-1}+a_2z^{-2}}
        \end{equation}
      $$
```

This shows 6 coefficients instead of 5 so, depending on your architecture, you will likely normalize \( a_0 \) to be 1 and perhaps also \(b_0\) to 1 (and collect that into an overall gain coefficient). Then your transfer function would look like:

```
      $$
        \begin{equation}
          H(z) = \frac{\left(\displaystyle\frac{b_0}{a_0}\right) + \left(\displaystyle\frac{b_1}{a_0}\right)z^{-1}+\left(\displaystyle\frac{b_2}{a_0}\right)z^{-2}}{1 + \left(\displaystyle\frac{a_1}{a_0}\right)z^{-1}+\left(\displaystyle\frac{a_2}{a_0}\right)z^{-2}}
          \label{direct-form-1}
        \end{equation}
      $$
```

or:

```
      $$
        \begin{equation}
          H(z) = \left(\frac{b_0}{a_0}\right) \frac{1 + \left(\displaystyle\frac{b_1}{b_0}\right)z^{-1}+\left(\displaystyle\frac{b_2}{b_0}\right)z^{-2}}{1 + \left(\displaystyle\frac{a_1}{a_0}\right)z^{-1}+\left(\displaystyle\frac{a_2}{a_0}\right)z^{-2}}
        \end{equation}
      $$
```

The most straight forward implementation would be the "Direct Form 1" (\(eq. \ref{direct-form-1}\)):

```
      $$
        \begin{align}
        y[n] = \left(\frac{b_0}{a_0}\right)x[n] & +
        \left(\frac{b_1}{a_0}\right)x[n-1] + \left(\frac{b_2}{a_0}\right)x[n-2]
        \nonumber \\
        &-\left(\frac{a_1}{a_0}\right)y[n-1] - \left(\frac{a_2}{a_0}\right)y[n-2]
        \end{align}
      $$
```

This is probably both the best and the easiest method to implement in the 56K and other fixed-point or floating-point architectures with a double wide accumulator.

Begin with these user defined parameters:

\(F_s\) the sampling frequency \(f_0\) Center Frequency or Corner Frequency, or shelf midpoint frequency, depending on which filter type. The "significant frequency". "wherever it's happenin', man." \(\mathrm{dBgain}\) used only for peaking and shelving filters \(Q\) or \(\mathrm{BW}\) or \(S\) \(Q\) the EE kind of definition, except for peakingEQ in which \(A\cdot Q\) is the classic EE \(Q\). That adjustment in definition was made so that a boost of \(N\) dB followed by a cut of \(N\) dB for identical \(Q\) and \(f_0/F_s\) results in a precisely flat unity gain filter or "wire". \(\mathrm{BW}\) the bandwidth in octaves (between -3 dB frequencies for BPF and notch or between midpoint (\(\mathrm{dBgain}/2\)) gain frequencies for peaking EQ) \(S\) a "shelf slope" parameter (for shelving EQ only). When \(S = 1\), the shelf slope is as steep as it can be and remain monotonically increasing or decreasing gain with frequency. The shelf slope, in dB/octave, remains proportional to \(S\) for all other values for a fixed \(f_0/F_s\) and \(\mathrm{dBgain}\).

Then compute a few intermediate variables:

1. Only for peaking and shelving EQ filters: `$$ \begin{align} A &= \sqrt{10^{\mathrm{dBgain}/20}} & \\ & = 10^{\mathrm{dBgain}/40} & \end{align} $$`
2. `$$ \omega_0 = 2 \pi \frac{f_0}{F_s} $$`
3. `$$ \begin{align*} & \cos \omega_0 \\ & \sin \omega_0 \end{align*} $$`
4. `$$ \begin{align} \alpha &= \frac{\sin \omega_0}{2 Q} & \textrm{(case: Q)} \\ &= \sin \omega_0 \, \sinh\left(\frac{\log 2}{2} \cdot \mathrm{BW} \cdot \frac{\omega_0}{\sin \omega_0}\right) & \textrm{(case: BW)} \\ &= \frac{\sin \omega_0}{2} \sqrt{\left(A + \frac{1}{A}\right) \left(\frac{1}{S} - 1\right) + 2} & \textrm{(case: S)} \end{align} $$`

FYI: The relationship between bandwidth and \(Q\) is

digital filter with BLT

```
      $$
        \begin{equation}
          \frac{1}{Q} = 2\sinh\left(\frac{\log 2}{2} \cdot \mathrm{BW} \cdot
          \frac{\omega_0}{\sin \omega_0}\right)
        \end{equation}
      $$
```

or

analog filter prototype

```
      $$
        \begin{equation}
          \frac{1}{Q} = 2 \sinh\left(\frac{\log 2}{2} \cdot \mathrm{BW}\right)
        \end{equation}
      $$
```

The relationship between shelf slope, \(S\), and \(Q\) is

```
      $$
        \begin{equation}
          \frac{1}{Q} = \sqrt{\left(A + \frac{1}{A}\right) \left(\frac{1}{S} -
          1\right) + 2}
        \end{equation}
      $$
```

```
      $$
        \begin{equation}
          2\sqrt{A}\,\alpha = (\sin \omega_0)\, \sqrt{\left(A^2 +
          1\right)\left(\frac{1}{S}-1\right) + 2A}
        \end{equation}
      $$
```

is a handy intermediate variable for shelving EQ filters.

Finally, compute the coefficients for whichever filter type you want:

(The analog prototypes, H(s), are shown for each filter type for normalized frequency.)

LPF

```
          $$
            \begin{equation}
              H(s) = \frac{1}{s^2 + \displaystyle\frac{s}{Q} + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &= \frac{1-\cos\omega_0}{2} \\
                b_1 &= 1-\cos\omega_0 \\
                b_2 &= \frac{1-\cos\omega_0}{2} \\
                a_0 &= 1 + \alpha \\
                a_1 &= -2\cos\omega_0 \\
                a_2 &= 1 - \alpha
              \end{split}
            \end{align}
          $$
```

HPF

```
          $$
            \begin{equation}
              H(s) = \frac{s^2}{s^2 + \displaystyle\frac{s}{Q} + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=  \frac{1 + \cos \omega_0}{2} \\
                b_1 &= -(1 + \cos \omega_0) \\
                b_2 &=  \frac{1 + \cos \omega_0}{2} \\
                a_0 &=   1 + \alpha \\
                a_1 &=  -2\cos \omega_0 \\
                a_2 &=   1 - \alpha
              \end{split}
            \end{align}
          $$
```

BPF
(constant skirt gain,
peak gain = \(Q\))

```
          $$
            \begin{equation}
              H(s) = \frac{s}{s^2 + \displaystyle\frac{s}{Q} + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=   \frac{\sin\omega_0}{2}  =   Q \alpha \\
                b_1 &=   0 \\
                b_2 &=  -\frac{\sin\omega_0}{2}  =  -Q \alpha \\
                a_0 &=   1 + \alpha \\
                a_1 &=  -2\cos\omega_0 \\
                a_2 &=   1 - \alpha
              \end{split}
            \end{align}
          $$
```

BPF
(constant 0 dB peak gain)

```
          $$
            \begin{equation}
              H(s) = \frac{\displaystyle\frac{s}{Q}}{s^2 + \displaystyle\frac{s}{Q} + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=   \alpha \\
                b_1 &=   0 \\
                b_2 &=  -\alpha \\
                a_0 &=   1 + \alpha \\
                a_1 &=  -2\cos \omega_0 \\
                a_2 &=   1 - \alpha
              \end{split}
            \end{align}
          $$
```

notch

```
          $$
            \begin{equation}
              H(s) = \frac{s^2 + 1}{s^2 + \displaystyle\frac{s}{Q} + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=   1 \\
                b_1 &=  -2\cos \omega_0 \\
                b_2 &=   1 \\
                a_0 &=   1 + \alpha \\
                a_1 &=  -2\cos \omega_0 \\
                a_2 &=   1 - \alpha
              \end{split}
            \end{align}
          $$
```

APF

```
          $$
            \begin{equation}
              H(s) = \frac{s^2 - \displaystyle\frac{s}{Q} + 1}{s^2 + \displaystyle\frac{s}{Q} + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=   1 - \alpha \\
                b_1 &=  -2\cos\omega_0 \\
                b_2 &=   1 + \alpha \\
                a_0 &=   1 + \alpha \\
                a_1 &=  -2\cos\omega_0 \\
                a_2 &=   1 - \alpha
              \end{split}
            \end{align}
          $$
```

peakingEQ

```
          $$
            \begin{equation}
              H(s) = \frac{s^2 + s\displaystyle\frac{A}{Q} + 1}{s^2 + \displaystyle\frac{s}{AQ} + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=   1 + \alpha A \\
                b_1 &=  -2\cos\omega_0 \\
                b_2 &=   1 - \alpha A \\
                a_0 &=   1 + \frac{\alpha}{A} \\
                a_1 &=  -2\cos\omega_0 \\
                a_2 &=   1 - \frac{\alpha}{A}
              \end{split}
            \end{align}
          $$
```

lowShelf

```
          $$
            \begin{equation}
              H(s) = A \frac{s^2 + \displaystyle\frac{\sqrt{A}}{Q} s + A}
                            {As^2 + \displaystyle\frac{\sqrt{A}}{Q} s + 1}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=    A\left( (A+1) - (A-1)\cos\omega_0 + 2\sqrt{A}\, \alpha \right) \\
                b_1 &=  2A\Big( (A-1) - (A+1)\cos\omega_0                   \Big) \\
                b_2 &=    A\left( (A+1) - (A-1)\cos\omega_0 - 2\sqrt{A}\, \alpha \right) \\
                a_0 &=        (A+1) + (A-1)\cos\omega_0 + 2\sqrt{A}\, \alpha \\
                a_1 &=   -2\Big( (A-1) + (A+1)\cos\omega_0                   \Big) \\
                a_2 &=        (A+1) + (A-1)\cos\omega_0 - 2\sqrt{A}\, \alpha
              \end{split}
            \end{align}
          $$
```

highShelf

```
          $$
            \begin{equation}
              H(s) = A \frac{As^2 + \displaystyle\frac{\sqrt{A}}{Q}s + 1}
                            {s^2 + \displaystyle\frac{\sqrt{A}}{Q} s + A}
            \end{equation}
          $$
```

```
          $$
            \begin{align}
              \begin{split}
                b_0 &=    A\left( (A+1) + (A-1)\cos\omega_0 + 2\sqrt{A}\alpha \right) \\
                b_1 &= -2A\Big( (A-1) + (A+1)\cos\omega_0                   \Big) \\
                b_2 &=    A\left( (A+1) + (A-1)\cos\omega_0 - 2\sqrt{A}\alpha \right) \\
                a_0 &=        (A+1) - (A-1)\cos\omega_0 + 2\sqrt{A}\alpha \\
                a_1 &=    2\Big( (A-1) - (A+1)\cos\omega_0                   \Big) \\
                a_2 &=        (A+1) - (A-1)\cos\omega_0 - 2\sqrt{A}\alpha
              \end{split}
            \end{align}
          $$
```

FYI: The bilinear transform (with compensation for frequency warping) substitutes:

(normalized)

```
      $$
        \begin{equation}
          s \leftarrow \frac{1}{\tan\displaystyle\frac{\omega_0}{2}}
            \times \frac{1-z^{-1}}{1+z^{-1}}
        \end{equation}
      $$
```

and makes use of these trig identities:

1. `$$ \begin{equation} \tan\frac{\omega_0}{2} = \frac{\sin\omega_0}{1+\cos\omega_0} \end{equation} $$`
2. `$$ \begin{equation} \left(\tan\frac{\omega_0}{2}\right)^2 = \frac{1-\cos\omega_0}{1+\cos\omega_0} \end{equation} $$`

resulting in these substitutions:

1. `$$ \begin{equation} 1 \leftarrow \frac{1+\cos\omega_0}{1+\cos\omega_0} \times \frac{1+2z^{-1}+z^{-2}}{1+2z^{-1}+z^{-2}} \end{equation} $$`
2. `$$ \begin{align} s \leftarrow & \, \frac{1+\cos\omega_0}{\sin\omega_0} \times \frac{1-z^{-1}}{1+z^{-1}} \\ & = \frac{1+\cos\omega_0}{\sin\omega_0} \times \frac{1-z^{-2}}{1+2z^{-1}+z^{-2}} \end{align} $$`
3. `$$ \begin{equation} s^2 \leftarrow \frac{1+\cos\omega_0}{1-\cos\omega_0} \times \frac{1-2z^{-1}+z^{-2}}{1+2z^{-1}+z^{-2}} \end{equation} $$`

The factor:

```
      $$
        \begin{equation}
          \frac{1+\cos\omega_0}{1+2z^{-1}+z^{-2}}
        \end{equation}
      $$
```

is common to all terms in both numerator and denominator, can be factored out, and thus be left out in the substitutions above resulting in:

1. `$$ \begin{equation} 1 \leftarrow \frac{1+2z^{-1}+z^{-2}}{1+\cos\omega_0} \end{equation} $$`
2. `$$ \begin{equation} s \leftarrow \frac{1-z^{-2}}{\sin\omega_0} \end{equation} $$`
3. `$$ \begin{equation} s^2 \leftarrow \frac{1-2z^{-1}+z^{-2}}{1-\cos\omega_0} \end{equation} $$`

In addition, all terms, numerator and denominator, can be multiplied by a common \(\sin^2\omega_0\) factor, finally resulting in these substitutions:

1. `$$ \begin{equation} 1 \leftarrow (1 + 2z^{-1} + z^{-2}) (1 - \cos\omega_0) \end{equation} $$`
2. `$$ \begin{equation} s \leftarrow (1-z^{-2})\sin\omega_0 \end{equation} $$`
3. `$$ \begin{equation} s^2 \leftarrow (1 - 2z^{-1} + z^{-2}) (1 + \cos\omega_0) \end{equation} $$`
4. `$$ \begin{equation} 1 + s^2 \leftarrow 2\, (1 - 2\cos\omega_0\,z^{-1} + z^{-2}) \end{equation} $$`

The biquad coefficient formulae above come out after a little simplification.

## Acknowledgements

Special thanks to [Robert Bristow-Johnson](mailto:rbj@audioimagination.com) for creating the Audio EQ Cookbook and permitting its adaption and use for the Web Audio API.

Thanks to [Peter Krautzberger](mailto:peter.krautzberger@mathjax.org) for help in adapting these mathematical formulae to MathML, and to the whole [MathJax](https://www.mathjax.org/) team for making the JavaScript extension that makes the use of math on the web possible.

Thanks to Peter Jipsen, creator of [ASCIIMathML](http://www1.chapman.edu/~jipsen/mathml/asciimath.html), for making it easier to convert ASCII math notation to MathML.

Converted to using TeX formulas instead of MathML by [Raymond Toy](mailto:toy.raymond@gmail.com).
