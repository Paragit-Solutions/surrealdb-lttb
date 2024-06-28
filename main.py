import numpy as np
import matplotlib.pyplot as plt
import argparse
from os import system
from lttb import downsample
import glob
import matplotlib.gridspec as gridspec


# Function to read int16 binary file
def read_int16_file(filename):
    with open(filename, 'rb') as f:
        buffer = f.read()
    data = np.frombuffer(buffer, dtype=np.int16)
    if len(data) % 6 != 0:
        raise ValueError("Data length is not a multiple of 6")
    data = data.reshape(-1, 6)
    return data


# Function to draw a chart
def draw_chart(ax, data, title, color):
    ax.plot(data[:, 0], data[:, 1], linestyle='-', color=color)
    ax.set_title(title)
    ax.grid(True)


# Parse arguments
parser = argparse.ArgumentParser(
    description="Downsample and visualize IMU data.")
parser.add_argument(
    "--column",
    type=str,
    default="ax",
    choices=["ax", "ay", "az", "gx", "gy", "gz"],
    help="Column to visualize (default: ax)",
)
args = parser.parse_args()

# Map column names to indices
column_map = {
    "ax": 0,
    "ay": 1,
    "az": 2,
    "gx": 3,
    "gy": 4,
    "gz": 5,
}
col_idx = column_map[args.column]

# Run the JavaScript code to generate downsampled data
system('./run-test.sh')

# Read the data
original_data = read_int16_file('data/motion.dat')
original_column_data = np.column_stack(
    (np.arange(len(original_data)), original_data[:, col_idx]))

# Find all downsampled files from both JavaScript and Python
js_files = sorted(glob.glob('data/downsampled_*.dat'))
py_ratios = [0.8, 0.5, 0.2, 0.1, 0.05, 0.01]

# Read JavaScript downsampled data
js_data = {
    float(file.split('_')[1].split('.')[0]) / 100:
    read_int16_file(file)[:, col_idx]
    for file in js_files
}

# Downsample using Python
py_data = {}
for ratio in py_ratios:
    size = max(2, int(len(original_column_data) * ratio))
    py_data[ratio] = downsample(original_column_data, size)

# Ensure correct format for JavaScript data
js_data = {
    k: np.column_stack((np.arange(len(v)), v))
    for k, v in js_data.items()
}

# Set up the plot with GridSpec
fig = plt.figure(figsize=(18, 25))
gs = gridspec.GridSpec(len(py_ratios) + 1,
                       2,
                       height_ratios=[2] + [1] * len(py_ratios))

# Draw the original data spanning the top two columns
ax0 = fig.add_subplot(gs[0, :])
draw_chart(ax0, original_column_data, "Original Data", 'red')

# Draw charts for downsampled data
for i, ratio in enumerate(py_ratios):
    ax_js = fig.add_subplot(gs[i + 1, 0])
    ax_py = fig.add_subplot(gs[i + 1, 1])
    title_js = f"JavaScript {ratio * 100:.0f}% ({len(js_data[ratio])} points)"
    title_py = f"Python {ratio * 100:.0f}% ({len(py_data[ratio])} points)"
    draw_chart(ax_js, js_data[ratio], title_js, 'green')
    draw_chart(ax_py, py_data[ratio], title_py, 'blue')

# Adjust layout to make sure everything fits well
plt.tight_layout()
plt.show()

# Save the plot as a PNG file
fig.savefig('test-output-plot.png')
