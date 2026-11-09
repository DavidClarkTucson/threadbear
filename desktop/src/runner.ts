// The runner, as the interface sees it: calls go to the Rust side, which
// owns the connection and the token (src-tauri/src/runner.rs), and what the
// runner pushes comes back as Tauri events.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Account, Pool, RunnerDefaults } from '../../runner/src/protocol.ts';

export type Hello = { runnerVersion: string; protocol: number; accounts: Account[]; pools?: Pool[]; defaults: RunnerDefaults };

export type Status =
  | { state: 'connecting'; url: string }
  | { state: 'connected'; url: string; hello: Hello }
  | { state: 'disconnected'; url: string; reason: string; retryInMs: number };

export type RpcError = { code: number; message: string; data?: unknown };

export type Notification = { method: string; params: any };

export function call<T = unknown>(method: string, params: Record<string, unknown> = {}): Promise<T> {
  return invoke<T>('runner_call', { method, params });
}

export function currentStatus(): Promise<Status> {
  return invoke<Status>('runner_status');
}

export function onStatus(f: (s: Status) => void): Promise<UnlistenFn> {
  return listen<Status>('runner-status', (e) => f(e.payload));
}

export function onNotification(f: (n: Notification) => void): Promise<UnlistenFn> {
  return listen<Notification>('runner-notification', (e) => f(e.payload));
}

export function errorText(e: unknown): string {
  if (e && typeof e === 'object' && 'message' in e) return String((e as RpcError).message);
  return String(e);
}
