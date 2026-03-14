import { type ChartConfig, ChartContainer } from '@shadcn';
import { Bar, BarChart, CartesianGrid } from 'recharts';

const chartData = [
  { month: 'January', success: 186, error: 80 },
  { month: 'February', success: 305, error: 200 },
  { month: 'March', success: 237, error: 120 },
  { month: 'April', success: 73, error: 190 },
  { month: 'May', success: 209, error: 130 },
  { month: 'June', success: 214, error: 140 },
];

const chartConfig = {
  success: {
    label: 'Success',
    color: '#00ac95',
  },
  error: {
    label: 'Error',
    color: '#ee8080',
  },
} satisfies ChartConfig;

export const PipelineChart = () => {
  return (
    <ChartContainer config={chartConfig} className='h-[100px] w-full'>
      <BarChart accessibilityLayer data={chartData}>
        <CartesianGrid vertical={false} />
        <Bar dataKey='success' fill='var(--color-success)' radius={4} />
        <Bar dataKey='error' fill='var(--color-error)' radius={4} />
      </BarChart>
    </ChartContainer>
  );
};
