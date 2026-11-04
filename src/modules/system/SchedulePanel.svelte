<script lang="ts">
  import { onMount } from 'svelte';
  import { errorMessage } from '../../core/api';
  import { t as shellT } from '../../core/i18n';
  import { toast } from '../../core/toast.svelte';
  import Button from '../../ui/Button.svelte';
  import Segmented from '../../ui/Segmented.svelte';
  import type { PlannedAction } from './api';
  import { t } from './i18n';
  import { planner } from './planner.svelte';
  import { clockTime, formatDuration, isTomorrow, minutesUntil, targetTime } from './times';

  const ACTIONS: PlannedAction[] = ['shutdown', 'restart', 'sleep'];
  const PRESETS = [15, 30, 60, 120];

  let now = $state(Date.now());
  onMount(() => {
    const timer = setInterval(() => (now = Date.now()), 15_000);
    return () => clearInterval(timer);
  });

  const draft = $derived(planner.draft);
  const target = $derived(targetTime(draft.mode, draft.minutes, draft.clock, new Date(now)));

  async function plan() {
    try {
      const at = await planner.plan();
      toast.show(
        t('schedule.done', {
          action: draft.action,
          time: clockTime(at),
          tomorrow: isTomorrow(at) ? 'yes' : 'no',
        }),
      );
    } catch (error) {
      toast.show(shellT('errors.generic', { message: errorMessage(error) }));
    }
  }

  async function cancel() {
    await planner.cancel();
    toast.show(t('schedule.cancelled'));
  }
</script>

<div class="panel">
  {#if planner.current}
    {@const current = planner.current}
    <div class="planned">
      <span class="label">{t('schedule.plannedAt', { action: current.action })}</span>
      <span class="time">{clockTime(current.at)}</span>
      <span class="label"
        >{t('schedule.left', { duration: formatDuration(minutesUntil(current.at, now)) })}</span
      >
    </div>
    <Button onclick={cancel}>{t('schedule.cancel')}</Button>
  {:else}
    <Segmented
      label={t('schedule.name')}
      options={ACTIONS.map((value) => ({ value, label: t(`schedule.actions.${value}`) }))}
      value={draft.action}
      onchange={(action) => (planner.draft.action = action)}
    />
    <Segmented
      label={t('schedule.name')}
      options={[
        { value: 'in' as const, label: t('schedule.inMode') },
        { value: 'at' as const, label: t('schedule.atMode') },
      ]}
      value={draft.mode}
      onchange={(mode) => (planner.draft.mode = mode)}
    />
    {#if draft.mode === 'in'}
      <div class="row">
        <Segmented
          label={t('schedule.minutes')}
          options={PRESETS.map((value) => ({ value: String(value), label: formatDuration(value) }))}
          value={String(draft.minutes)}
          onchange={(value) => (planner.draft.minutes = Number(value))}
        />
        <label class="minutes">
          <input
            type="number"
            min="1"
            max="1440"
            aria-label={t('schedule.minutes')}
            value={draft.minutes}
            oninput={(event) =>
              (planner.draft.minutes = Math.max(1, Math.min(1440, Number(event.currentTarget.value) || 1)))}
          />
          min
        </label>
      </div>
    {:else}
      <label class="clock">
        {t('schedule.time')}
        <input
          type="time"
          value={draft.clock}
          oninput={(event) => (planner.draft.clock = event.currentTarget.value)}
        />
      </label>
    {/if}
    <p>
      {t('schedule.summary', {
        action: draft.action,
        time: clockTime(target),
        tomorrow: isTomorrow(target, new Date(now)) ? 'yes' : 'no',
        duration: formatDuration(minutesUntil(target.getTime(), now)),
      })}
    </p>
    <Button variant="primary" onclick={plan}>{t('schedule.submit')}</Button>
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 14px;
  }

  .planned {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .label {
    font-size: 13px;
    color: var(--text-muted);
  }

  .time {
    font-size: 38px;
    font-weight: 650;
    letter-spacing: -0.01em;
    font-variant-numeric: tabular-nums;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  .minutes,
  .clock {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-muted);
  }

  input {
    height: 32px;
    padding: 0 8px;
    border-radius: 8px;
    background: var(--surface-sunken);
    border: 1px solid var(--border-strong);
    font: inherit;
    font-size: 14px;
    color: var(--text);
  }

  .minutes input {
    width: 66px;
  }

  p {
    font-size: 13.5px;
    line-height: 1.5;
  }
</style>
