DEFINE FUNCTION fn::lttb($data: array<number>, $n_out: int) {
    RETURN function($data, $n_out) {
        const [data, n_out] = arguments;
        const n = data.length;
        if (n_out >= n || n_out === 0) {
          return data.map((value, index) => [index, value]); // Return all points with indices
        }

        const sampled = new Array(n_out);
        const bucketSize = (n - 2) / (n_out - 2);

        let a = 0;
        sampled[0] = [0, data[0]]; // Always add the first point with its index
        sampled[n_out - 1] = [n - 1, data[n - 1]]; // Always add the last point with its index

        for (let i = 1; i < n_out - 1; i++) {
          const nextA = Math.floor((i + 1) * bucketSize) + 1;
          const avgRangeStart = Math.floor(i * bucketSize) + 1;
          const avgRangeEnd = nextA < n ? nextA : n;

          let avg = 0;

          for (let j = avgRangeStart; j < avgRangeEnd; j++) {
            avg += data[j];
          }
          const avgRangeLength = avgRangeEnd - avgRangeStart;
          avg /= avgRangeLength;

          const rangeOffs = Math.floor((i - 1) * bucketSize) + 1;
          const rangeTo = Math.floor(i * bucketSize) + 1;

          let maxArea = -1;
          let maxAreaPoint = a;

          const pointA = data[a];

          for (let j = rangeOffs; j < rangeTo; j++) {
            const area = Math.abs(
              (pointA - avg) * (data[j] - pointA) -
              (pointA - data[j]) * (avg - pointA)
            );

            if (area > maxArea) {
              maxArea = area;
              maxAreaPoint = j;
            }
          }

          sampled[i] = [maxAreaPoint, data[maxAreaPoint]];
          a = maxAreaPoint;
        }

        return sampled;
      };
}
