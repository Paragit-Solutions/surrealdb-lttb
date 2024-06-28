import numpy as np
import matplotlib.pyplot as plt
import argparse
from lttb import downsample
import glob
import matplotlib.gridspec as gridspec


# Function to read int16 binary file
def read_int16_file(filename):
    with open(filename, 'rb') as f:
        buffer = f.read()
    data = np.frombuffer(buffer, dtype=np.int16)
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

# Read the original data
original_data = read_int16_file('data/motion.dat')
original_column_data = np.column_stack(
    (np.arange(len(original_data)), original_data[:, col_idx]))

# Find all downsampled files
downsampled_files = sorted(glob.glob(f'data/motion-*.dat'))
downsampled_data = {
    int(file.split('-')[-1].split('.')[0]): read_int16_file(file)[:, col_idx]
    for file in downsampled_files
}

# Downsample using Python LTTB implementation
py_ratios = [80, 50, 20, 10, 5, 1]
py_data = {}
for ratio in py_ratios:
    size = max(2, int(len(original_column_data) * ratio / 100))
    py_data[ratio] = downsample(original_column_data, size)

# Ensure correct format for JavaScript data
downsampled_data = {
    k: np.column_stack((np.arange(len(v)), v))
    for k, v in downsampled_data.items()
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
    title_js = f"JavaScript {ratio}% ({len(downsampled_data[ratio])} points)"
    title_py = f"Python {ratio}% ({len(py_data[ratio])} points)"
    draw_chart(ax_js, downsampled_data[ratio], title_js, 'green')
    draw_chart(ax_py, py_data[ratio], title_py, 'blue')

# Adjust layout to make sure everything fits well
plt.tight_layout()
plt.show()

# Save the plot as a PNG file
fig.savefig('test-output-plot.png')
