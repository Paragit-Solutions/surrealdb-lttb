DEFINE FUNCTION fn::lttb($data: array<object>, $threshold: int, $columns: array<string>) {
    RETURN SELECT * FROM function($data, $threshold, $columns) {
        const [data, threshold, columns] = arguments;
        const n = data.length;
        if (threshold >= n || threshold === 0) {
          return data; // Nothing to do
        }

        const sampled = new Array(threshold);
        const bucketSize = (n - 2) / (threshold - 2);

        let a = 0;
        sampled[0] = data[0]; // Always add the first point
        sampled[threshold - 1] = data[n - 1]; // Always add the last point

        for (let i = 1; i < threshold - 1; i++) {
          const nextA = Math.floor((i + 1) * bucketSize) + 1;
          const avgRangeStart = Math.floor(i * bucketSize) + 1;
          const avgRangeEnd = nextA < n ? nextA : n;

          const avg = {};
          columns.forEach((col) => {
            avg[col] = 0;
          });

          for (let j = avgRangeStart; j < avgRangeEnd; j++) {
            columns.forEach((col) => {
              avg[col] += data[j][col];
            });
          }
          const avgRangeLength = avgRangeEnd - avgRangeStart;
          columns.forEach((col) => {
            avg[col] /= avgRangeLength;
          });

          const rangeOffs = Math.floor((i - 1) * bucketSize) + 1;
          const rangeTo = Math.floor(i * bucketSize) + 1;

          let maxArea = -1;
          let maxAreaPoint = a;

          const pointA = {};
          columns.forEach((col) => {
            pointA[col] = data[a][col];
          });

          for (let j = rangeOffs; j < rangeTo; j++) {
            const area = columns.reduce((acc, col) => {
              return (
                acc +
                Math.abs(
                  (pointA[col] - avg[col]) * (data[j][col] - pointA[col]) -
                    (pointA[col] - data[j][col]) * (avg[col] - pointA[col])
                )
              );
            }, 0);

            if (area > maxArea) {
              maxArea = area;
              maxAreaPoint = j;
            }
          }

          sampled[i] = data[maxAreaPoint];
          a = maxAreaPoint;
        }

        return sampled;
      };
}
