DEFINE FUNCTION fn::lttb_fields($data: object, $n_out: int, $fields: array<string>) {
    RETURN function($data, $n_out, $fields) {
        const [data, n_out, fields] = arguments;
        const result = {};

        fields.forEach(field => {
            const values = data[field];
            const n = values.length;
            if (n_out >= n || n_out === 0) {
                result[field] = values.map((value, index) => [index, value]);
                return;
            }

            const sampled = new Array(n_out);
            const bucketSize = (n - 2) / (n_out - 2);

            let a = 0;
            sampled[0] = [0, values[0]];
            sampled[n_out - 1] = [n - 1, values[n - 1]];

            for (let i = 1; i < n_out - 1; i++) {
                const nextA = Math.floor((i + 1) * bucketSize) + 1;
                const avgRangeStart = Math.floor(i * bucketSize) + 1;
                const avgRangeEnd = nextA < n ? nextA : n;

                let avg = 0;
                for (let j = avgRangeStart; j < avgRangeEnd; j++) {
                    avg += values[j];
                }
                avg /= (avgRangeEnd - avgRangeStart);

                const rangeOffs = Math.floor((i - 1) * bucketSize) + 1;
                const rangeTo = Math.floor(i * bucketSize) + 1;

                let maxArea = -1;
                let maxAreaPoint = a;

                const pointA = values[a];

                for (let j = rangeOffs; j < rangeTo; j++) {
                    const area = Math.abs(
                        (pointA - avg) * (values[j] - pointA) -
                        (pointA - values[j]) * (avg - pointA)
                    );

                    if (area > maxArea) {
                        maxArea = area;
                        maxAreaPoint = j;
                    }
                }

                sampled[i] = [maxAreaPoint, values[maxAreaPoint]];
                a = maxAreaPoint;
            }

            result[field] = sampled;
        });

        return result;
    };
}
