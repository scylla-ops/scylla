"use client"

import * as React from "react"
import { Area, AreaChart, CartesianGrid, XAxis } from "recharts"

import {
    Card,
    CardAction,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/modules/core/presentation/ui/shadcn"
import {
    type ChartConfig,
    ChartContainer,
    ChartTooltip,
    ChartTooltipContent,
} from "@/modules/core/presentation/ui/shadcn"
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/modules/core/presentation/ui/shadcn"
import {
    ToggleGroup,
    ToggleGroupItem,
} from "@/modules/core/presentation/ui/shadcn"

export const description = "An interactive area chart"

const chartData = [
  { date: "2024-04-01", data: 222 },
  { date: "2024-04-02", data: 97 },
  { date: "2024-04-03", data: 167 },
  { date: "2024-04-04", data: 242 },
  { date: "2024-04-05", data: 373 },
  { date: "2024-04-06", data: 301 },
  { date: "2024-04-07", data: 245 },
  { date: "2024-04-08", data: 409 },
  { date: "2024-04-09", data: 59 },
  { date: "2024-04-10", data: 261 },
  { date: "2024-04-11", data: 327 },
  { date: "2024-04-12", data: 292 },
  { date: "2024-04-13", data: 342 },
  { date: "2024-04-14", data: 137 },
  { date: "2024-04-15", data: 120 },
  { date: "2024-04-16", data: 138 },
  { date: "2024-04-17", data: 446 },
  { date: "2024-04-18", data: 364 },
  { date: "2024-04-19", data: 243 },
  { date: "2024-04-20", data: 89 },
  { date: "2024-04-21", data: 137 },
  { date: "2024-04-22", data: 224 },
  { date: "2024-04-23", data: 138 },
  { date: "2024-04-24", data: 387 },
  { date: "2024-04-25", data: 215 },
  { date: "2024-04-26", data: 75 },
  { date: "2024-04-27", data: 383 },
  { date: "2024-04-28", data: 122 },
  { date: "2024-04-29", data: 315 },
  { date: "2024-04-30", data: 454 },
  { date: "2024-05-01", data: 165 },
  { date: "2024-05-02", data: 293 },
  { date: "2024-05-03", data: 247 },
  { date: "2024-05-04", data: 385 },
  { date: "2024-05-05", data: 481 },
  { date: "2024-05-06", data: 498 },
  { date: "2024-05-07", data: 388 },
  { date: "2024-05-08", data: 149 },
  { date: "2024-05-09", data: 227 },
  { date: "2024-05-10", data: 293 },
  { date: "2024-05-11", data: 335 },
  { date: "2024-05-12", data: 197 },
  { date: "2024-05-13", data: 197 },
  { date: "2024-05-14", data: 448 },
  { date: "2024-05-15", data: 473 },
  { date: "2024-05-16", data: 338 },
  { date: "2024-05-17", data: 499 },
  { date: "2024-05-18", data: 315 },
  { date: "2024-05-19", data: 235 },
  { date: "2024-05-20", data: 177 },
  { date: "2024-05-21", data: 82 },
  { date: "2024-05-22", data: 81 },
  { date: "2024-05-23", data: 252 },
  { date: "2024-05-24", data: 294 },
  { date: "2024-05-25", data: 201 },
  { date: "2024-05-26", data: 213 },
  { date: "2024-05-27", data: 420 },
  { date: "2024-05-28", data: 233 },
  { date: "2024-05-29", data: 78 },
  { date: "2024-05-30", data: 340 },
  { date: "2024-05-31", data: 178 },
  { date: "2024-06-01", data: 178 },
  { date: "2024-06-02", data: 470 },
  { date: "2024-06-03", data: 103 },
  { date: "2024-06-04", data: 439 },
  { date: "2024-06-05", data: 88 },
  { date: "2024-06-06", data: 294 },
  { date: "2024-06-07", data: 323 },
  { date: "2024-06-08", data: 385 },
  { date: "2024-06-09", data: 438 },
  { date: "2024-06-10", data: 155 },
  { date: "2024-06-11", data: 92 },
  { date: "2024-06-12", data: 492 },
  { date: "2024-06-13", data: 81 },
  { date: "2024-06-14", data: 426 },
  { date: "2024-06-15", data: 307 },
  { date: "2024-06-16", data: 371 },
  { date: "2024-06-17", data: 475 },
  { date: "2024-06-18", data: 107 },
  { date: "2024-06-19", data: 341 },
  { date: "2024-06-20", data: 408 },
  { date: "2024-06-21", data: 169 },
  { date: "2024-06-22", data: 317 },
  { date: "2024-06-23", data: 480 },
  { date: "2024-06-24", data: 132 },
  { date: "2024-06-25", data: 141 },
  { date: "2024-06-26", data: 434 },
  { date: "2024-06-27", data: 448 },
  { date: "2024-06-28", data: 149 },
  { date: "2024-06-29", data: 103 },
  { date: "2024-06-30", data: 446 },
]

const chartConfig = {
  visitors: {
    label: "Visitors",
  },
  data: {
    label: "Pipelines",
    color: "var(--primary)",
  },
} satisfies ChartConfig

export const PipelineChart = () => {
  const [timeRange, setTimeRange] = React.useState("90d")

  const filteredData = chartData.filter((item) => {
    const date = new Date(item.date)
    const referenceDate = new Date("2024-06-30")
    let daysToSubtract = 90
    if (timeRange === "30d") {
      daysToSubtract = 30
    } else if (timeRange === "7d") {
      daysToSubtract = 7
    }
    const startDate = new Date(referenceDate)
    startDate.setDate(startDate.getDate() - daysToSubtract)
    return date >= startDate
  })

  return (
    <Card className="@container/card">
      <CardHeader>
        <CardTitle>Total Pipelines Executed</CardTitle>
        <CardDescription>
          <span className="hidden @[540px]/card:block">
            Total for the last 3 months
          </span>
          <span className="@[540px]/card:hidden">Last 3 months</span>
        </CardDescription>
        <CardAction>
          <ToggleGroup
            type="single"
            value={timeRange}
            onValueChange={setTimeRange}
            variant="outline"
            className="hidden *:data-[slot=toggle-group-item]:!px-4 @[767px]/card:flex"
          >
            <ToggleGroupItem value="90d">Last 3 months</ToggleGroupItem>
            <ToggleGroupItem value="30d">Last 30 days</ToggleGroupItem>
            <ToggleGroupItem value="7d">Last 7 days</ToggleGroupItem>
          </ToggleGroup>
          <Select value={timeRange} onValueChange={setTimeRange}>
            <SelectTrigger
              className="flex w-40 **:data-[slot=select-value]:block **:data-[slot=select-value]:truncate @[767px]/card:hidden"
              size="sm"
              aria-label="Select a value"
            >
              <SelectValue placeholder="Last 3 months" />
            </SelectTrigger>
            <SelectContent className="rounded-xl">
              <SelectItem value="90d" className="rounded-lg">
                Last 3 months
              </SelectItem>
              <SelectItem value="30d" className="rounded-lg">
                Last 30 days
              </SelectItem>
              <SelectItem value="7d" className="rounded-lg">
                Last 7 days
              </SelectItem>
            </SelectContent>
          </Select>
        </CardAction>
      </CardHeader>
      <CardContent className="px-2 pt-4 sm:px-6 sm:pt-6">
        <ChartContainer
          config={chartConfig}
          className="aspect-auto h-[250px] w-full"
        >
          <AreaChart data={filteredData}>
            <defs>
              <linearGradient id="fillDesktop" x1="0" y1="0" x2="0" y2="1">
                <stop
                  offset="5%"
                  stopColor="var(--color-primary)"
                  stopOpacity={1.0}
                />
                <stop
                  offset="95%"
                  stopColor="var(--color-primary)"
                  stopOpacity={0.1}
                />
              </linearGradient>
            </defs>
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="date"
              tickLine={false}
              axisLine={false}
              tickMargin={8}
              minTickGap={32}
              tickFormatter={(value) => {
                const date = new Date(value)
                return date.toLocaleDateString("en-US", {
                  month: "short",
                  day: "numeric",
                })
              }}
            />
            <ChartTooltip
              cursor={false}
              content={
                <ChartTooltipContent
                  labelFormatter={(value) => {
                    return new Date(value).toLocaleDateString("en-US", {
                      month: "short",
                      day: "numeric",
                    })
                  }}
                  indicator="dot"
                />
              }
            />
            <Area
              dataKey="data"
              type="natural"
              fill="url(#fillDesktop)"
              stroke="var(--color-primary)"
              stackId="a"
            />
          </AreaChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}
