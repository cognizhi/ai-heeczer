export type ConfidenceBand = "High" | "Medium" | "Low";

export interface TrendPoint {
  date: string;
  tasks: number;
  hours: number;
  fec: number;
}

export interface ScoreRow {
  eventId: string;
  task: string;
  project: string;
  category: string;
  framework: string;
  team: string;
  user: string;
  minutes: number;
  fec: number;
  confidence: ConfidenceBand;
  timestamp: string;
  summary: string;
  trace: Array<{ label: string; value: string }>;
}

export interface LeaderboardRow {
  label: string;
  tasks: number;
  hours: number;
  fec: number;
  confidence: ConfidenceBand;
}

export interface QueueMetric {
  label: string;
  value: string;
  state: "good" | "warn" | "bad";
}

const scores: ScoreRow[] = [
  {
    eventId: "00000000-0000-4000-8000-000000000101",
    task: "PRD section synthesis",
    project: "platform-roi",
    category: "summarization",
    framework: "langgraph",
    team: "AI Enablement",
    user: "u-alina",
    minutes: 64,
    fec: 96,
    confidence: "High",
    timestamp: "2026-04-25T08:35:00Z",
    summary: "Structured source material into release-ready product language.",
    trace: [
      { label: "Base cognitive units", value: "42.00" },
      { label: "Task category multiplier", value: "summarization / 1.15" },
      { label: "Review adjustment", value: "human-in-loop present" },
      { label: "Confidence", value: "High from complete telemetry" },
    ],
  },
  {
    eventId: "00000000-0000-4000-8000-000000000102",
    task: "CI failure triage",
    project: "sdk-parity",
    category: "root_cause_analysis",
    framework: "google_adk",
    team: "SDK",
    user: "u-mika",
    minutes: 38,
    fec: 57,
    confidence: "Medium",
    timestamp: "2026-04-25T09:15:00Z",
    summary: "Clustered failed jobs and identified a contract drift candidate.",
    trace: [
      { label: "Base cognitive units", value: "31.00" },
      { label: "Risk class", value: "medium" },
      { label: "Retries", value: "1 retry penalty" },
      { label: "Confidence", value: "Medium from partial context" },
    ],
  },
  {
    eventId: "00000000-0000-4000-8000-000000000103",
    task: "Queue backlog report",
    project: "ingestion-service",
    category: "planning",
    framework: "cli",
    team: "Platform",
    user: "u-ren",
    minutes: 24,
    fec: 36,
    confidence: "Low",
    timestamp: "2026-04-24T16:40:00Z",
    summary: "Estimated queue delay impact from sparse worker telemetry.",
    trace: [
      { label: "Base cognitive units", value: "20.00" },
      { label: "Telemetry completeness", value: "missing queue age p95" },
      { label: "Confidence", value: "Low until worker metrics ship" },
    ],
  },
];

const trends: TrendPoint[] = [
  { date: "Apr 19", tasks: 18, hours: 9.4, fec: 846 },
  { date: "Apr 20", tasks: 21, hours: 12.1, fec: 1089 },
  { date: "Apr 21", tasks: 17, hours: 8.7, fec: 783 },
  { date: "Apr 22", tasks: 34, hours: 19.8, fec: 1782 },
  { date: "Apr 23", tasks: 29, hours: 16.2, fec: 1458 },
  { date: "Apr 24", tasks: 26, hours: 14.6, fec: 1314 },
  { date: "Apr 25", tasks: 31, hours: 18.9, fec: 1701 },
];

function groupBy(
  key: keyof Pick<
    ScoreRow,
    "team" | "project" | "framework" | "category" | "user"
  >,
): LeaderboardRow[] {
  const rows = new Map<
    string,
    { tasks: number; minutes: number; fec: number; confidence: ConfidenceBand }
  >();
  for (const score of scores) {
    const label = score[key];
    const current = rows.get(label) ?? {
      tasks: 0,
      minutes: 0,
      fec: 0,
      confidence: score.confidence,
    };
    current.tasks += 1;
    current.minutes += score.minutes;
    current.fec += score.fec;
    current.confidence =
      current.confidence === "Low" || score.confidence === "Low"
        ? "Low"
        : score.confidence === "Medium"
          ? "Medium"
          : current.confidence;
    rows.set(label, current);
  }
  return [...rows.entries()]
    .map(([label, row]) => ({
      label,
      tasks: row.tasks,
      hours: Number((row.minutes / 60).toFixed(1)),
      fec: row.fec,
      confidence: row.confidence,
    }))
    .sort((a, b) => b.fec - a.fec);
}

export async function getDashboardData() {
  const totalMinutes = scores.reduce((sum, score) => sum + score.minutes, 0);
  const totalFec = scores.reduce((sum, score) => sum + score.fec, 0);
  return {
    summary: {
      totalTasks: scores.length,
      hours: Number((totalMinutes / 60).toFixed(1)),
      days: Number((totalMinutes / 480).toFixed(2)),
      fec: totalFec,
      confidence: {
        High: scores.filter((score) => score.confidence === "High").length,
        Medium: scores.filter((score) => score.confidence === "Medium").length,
        Low: scores.filter((score) => score.confidence === "Low").length,
      },
    },
    trends,
    scores,
    leaderboards: {
      team: groupBy("team"),
      project: groupBy("project"),
      framework: groupBy("framework"),
      category: groupBy("category"),
      user: groupBy("user"),
    },
    queue: [
      { label: "Pending", value: "18", state: "warn" as const },
      { label: "Oldest age", value: "4m 12s", state: "good" as const },
      { label: "Throughput", value: "840/min", state: "good" as const },
      { label: "Retries", value: "3", state: "warn" as const },
      { label: "DLQ", value: "0", state: "good" as const },
    ],
  };
}

export async function getScore(eventId: string): Promise<ScoreRow | null> {
  const data = await getDashboardData();
  return data.scores.find((score) => score.eventId === eventId) ?? null;
}
