const fs = require("fs");
const lttb = require("./lttb");

// Function to read int16 binary file and convert to objects
function readInt16File(filename) {
  const buffer = fs.readFileSync(filename);
  const int16Array = new Int16Array(
    buffer.buffer,
    buffer.byteOffset,
    buffer.length / 2
  );
  const data = [];
  for (let i = 0; i < int16Array.length; i += 6) {
    data.push({
      ax: int16Array[i],
      ay: int16Array[i + 1],
      az: int16Array[i + 2],
      gx: int16Array[i + 3],
      gy: int16Array[i + 4],
      gz: int16Array[i + 5],
    });
  }
  return data;
}

// Function to write int16 binary file from objects
function writeInt16File(filename, data) {
  const buffer = new ArrayBuffer(data.length * 6 * 2);
  const int16Array = new Int16Array(buffer);
  for (let i = 0; i < data.length; i++) {
    int16Array[i * 6] = data[i].ax;
    int16Array[i * 6 + 1] = data[i].ay;
    int16Array[i * 6 + 2] = data[i].az;
    int16Array[i * 6 + 3] = data[i].gx;
    int16Array[i * 6 + 4] = data[i].gy;
    int16Array[i * 6 + 5] = data[i].gz;
  }
  fs.writeFileSync(filename, Buffer.from(buffer));
}

// Read the data
const data = readInt16File("data/imu.dat");

// Validate data
if (!data || data.length === 0) {
  throw new Error("Data is empty or undefined");
}

// Calculate downsample sizes
const downsampleRatios = [0.8, 0.5, 0.2, 0.1, 0.05, 0.01];
const columns = ["ax", "ay", "az", "gx", "gy", "gz"];
const downsampledData = downsampleRatios.map((ratio) => {
  const size = Math.max(2, Math.floor(data.length * ratio));
  return ratio === 1 ? data : lttb(data, size, columns);
});

// Save downsampled data
downsampledData.forEach((data, index) => {
  const ratio = downsampleRatios[index];
  const filename = `data/downsampled_${Math.round(ratio * 100)}.dat`;
  writeInt16File(filename, data);
  console.log(`Downsampled data saved as ${filename}`);
});
