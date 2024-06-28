# Downsampling Comparison

This repository contains code for comparing downsampling techniques implemented in JavaScript and Python. The downsampling is performed on `int16` binary data, and the results are plotted for visual comparison.

## Overview

The main objectives of this repository are:

- Read `int16` binary data from a file.
- Downsample the data using both JavaScript and Python implementations.
- Plot the original and downsampled data for visual comparison.

### Key Features

- **Data Reading**: Efficiently reads `int16` binary data files.
- **Downsampling**: Utilizes the `lttb` (Largest-Triangle-Three-Buckets) algorithm for downsampling the data.
- **Visualization**: Plots the original and downsampled data using `matplotlib` for clear visual comparison.

## Plot

Below is the generated plot comparing the original data with the downsampled data at various ratios for both JavaScript and Python implementations.

![Downsampling Comparison](test-output-plot.png)

## How to Run

1. **Install Dependencies**:
   Ensure you have the necessary Python packages installed:
   ```bash
   pip install numpy matplotlib glob2 lttb
   ```
