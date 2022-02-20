multinortest
============

Performs a multinormality, that is a multivariate normality test,
on a HDF5 dataset, using Mardia's test.

This test computes two values A and B.

This gives you a clue about whether your data can be modeled
as a multinormal i.e. Gaussian distribution.

If the values are not within their expected ranges, the data
is unlikely to be Gaussian.

The data is an (m,n) array where each row is a vector of dimension n.
It is loaded from an HDF5 file.

Compilation
-----------

```
RUSTFLAGS="-C target-cpu=native" cargo build --release 
```

Usage
-----

- `--path` gives the path to your HDF5 dataset (you're using HDF5 to store your data,
aren't you?)
- `--name` name of the HDF5 variable, this should be a double-precision rank 2 array of dimensions `(m,n)` giving `m` sample vectors of dimension `n`
- `--range i0 i1` optional, restrict the data to rows `i0` to `i1` (zero-based indexing)
- `--simulate` replace the data with data having the same effective dimensions, mean and sample covariance for checking the test

Example usage
-------------

```
% target/release/multinortest --path samples.h5 --name x --range 0 160
HDF5 path:    samples.h5
Dataset name: x
Dimensions:   3968 by 39
Range:        0 to 160
Eff. dims.:   160 by 39
A : got 18815.5, expected 10660.0 plus or minus 146.014
B : got 8.271, expected 0 plus or minus 1
```

See https://en.wikipedia.org/wiki/Multivariate_normal_distribution

Author
------

Berké DURAK <bd@exhrd.fr>

TODO
----

[ ] Parallelize using Rayon
[ ] Complain if there aren't enough samples
[ ] Implement the other tests (Henze-Zirkler, Royston, Q-Q plot...)
[ ] Option for transposing the variable
[ ] Option for selecting a random subset
