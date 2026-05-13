export interface ScanProgressEvent {
  source: string;
  status: "scanning" | "done";
  found: number;
}

export interface ApplyProgressEvent {
  step: string;
  current: number;
  total: number;
}
