/**
 * Maps job status from backend to chart status code
 * @param jobStatus - Status from JobResponse ("success", "failed", "running", "pending")
 * @returns Chart status code (0 = failed, 1 = success, 2 = running/pending)
 */
export const mapJobStatusToChartStatus = (jobStatus: string): number => {
  switch (jobStatus.toLowerCase()) {
    case 'success':
      return 1; // Green
    case 'failed':
      return 0; // Red
    case 'running':
    case 'pending':
      return 2; // Blue with animation
    default:
      return 1; // Default to success
  }
};

/**
 * Data point for the pipeline history chart
 */
export type ChartDataPoint = {
  jobId: string;
  status: number; // 0 = failed, 1 = success, 2 = running/pending
  createdAt: string;
  updatedAt: string;
  duration: number; // in seconds
  runNumber: number;
};

