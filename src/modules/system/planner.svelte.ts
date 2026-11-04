import { systemApi, type PlannedAction, type Schedule } from './api';
import { defaultClock, targetTime } from './times';

export interface Draft {
  action: PlannedAction;
  mode: 'in' | 'at';
  minutes: number;
  clock: string;
}

class PlannerState {
  current = $state<Schedule | null>(null);
  /** The choice being prepared in the panel; Enter in the search plans it too. */
  draft = $state<Draft>({ action: 'shutdown', mode: 'in', minutes: 30, clock: defaultClock() });
  #loading: Promise<void> | null = null;

  load(): Promise<void> {
    this.#loading ??= (async () => {
      this.current = await systemApi.currentSchedule();
      await systemApi.onSchedule((schedule) => (this.current = schedule));
    })();
    return this.#loading;
  }

  /** Plans the draft and returns the moment it was planned for. */
  async plan(): Promise<Date> {
    const { action, mode, minutes, clock } = this.draft;
    const at = targetTime(mode, minutes, clock);
    this.current = await systemApi.schedule({ action, at: at.getTime() });
    return at;
  }

  async cancel(): Promise<void> {
    await systemApi.cancelSchedule();
    this.current = null;
  }
}

export const planner = new PlannerState();
