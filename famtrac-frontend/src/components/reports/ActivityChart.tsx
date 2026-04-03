import { Card } from 'react-bootstrap';
import {
  ResponsiveContainer,
  LineChart,
  BarChart,
  XAxis,
  YAxis,
  Tooltip,
  Legend,
  Line,
  Bar,
} from 'recharts';

type ChartType = 'line' | 'bar';

export interface ChartDataPoint {
  label: string;
  value: number;
}

export interface StackedBarSeries {
  dataKey: string;
  color: string;
  name: string;
}

interface BaseChartProps {
  title: string;
  yAxisLabel: string;
  xAxisLabel: string;
  emptyMessage: string;
}

interface SimpleChartProps extends BaseChartProps {
  data: ChartDataPoint[];
  chartType: ChartType;
  color: string;
  stackedBars?: never;
}

interface StackedChartProps extends BaseChartProps {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  data: Record<string, any>[];
  chartType: 'stacked-bar';
  stackedBars: StackedBarSeries[];
  color?: never;
}

export type ActivityChartProps = SimpleChartProps | StackedChartProps;

/**
 * ActivityChart renders a line or bar chart for activity data using Recharts.
 * When data is empty, displays the emptyMessage instead of a chart.
 * - Uses Recharts for rendering (Requirement 8.1)
 * - Responsive charts that adapt to container width (Requirement 8.2)
 * - Displays axis labels for X and Y axes (Requirement 8.3)
 * - Displays tooltip on hover (Requirement 8.4)
 */
export function ActivityChart(props: ActivityChartProps) {
  const { title, data, chartType, yAxisLabel, xAxisLabel, emptyMessage } = props;

  const isEmpty = data.length === 0;

  return (
    <Card className="mb-3">
      <Card.Header>{title}</Card.Header>
      <Card.Body>
        {isEmpty ? (
          <p className="text-muted text-center my-4">{emptyMessage}</p>
        ) : (
          <ResponsiveContainer width="100%" height={300}>
            {chartType === 'line' ? (
              <LineChart data={data}>
                <XAxis
                  dataKey="label"
                  label={{ value: xAxisLabel, position: 'insideBottom', offset: -5 }}
                />
                <YAxis label={{ value: yAxisLabel, angle: -90, position: 'insideLeft' }} />
                <Tooltip />
                <Line type="monotone" dataKey="value" stroke={props.color} dot />
              </LineChart>
            ) : chartType === 'stacked-bar' ? (
              <BarChart data={data}>
                <XAxis
                  dataKey="label"
                  label={{ value: xAxisLabel, position: 'insideBottom', offset: -5 }}
                />
                <YAxis label={{ value: yAxisLabel, angle: -90, position: 'insideLeft' }} />
                <Tooltip />
                <Legend />
                {props.stackedBars.map((s) => (
                  <Bar
                    key={s.dataKey}
                    dataKey={s.dataKey}
                    stackId="a"
                    fill={s.color}
                    name={s.name}
                  />
                ))}
              </BarChart>
            ) : (
              <BarChart data={data}>
                <XAxis
                  dataKey="label"
                  label={{ value: xAxisLabel, position: 'insideBottom', offset: -5 }}
                />
                <YAxis label={{ value: yAxisLabel, angle: -90, position: 'insideLeft' }} />
                <Tooltip />
                <Bar dataKey="value" fill={props.color} />
              </BarChart>
            )}
          </ResponsiveContainer>
        )}
      </Card.Body>
    </Card>
  );
}
