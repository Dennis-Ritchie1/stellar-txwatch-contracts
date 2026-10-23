// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Tx-wat Contributors

import { Address, Contract, NativeToScenMap, Scval, XdriValue } from '@stellar/stellar-sdk';

export type AlertLevel = {
  tag: 'Info';
  values: void;
} | {
  tag: 'Warn';
  values: void;
} | {
  tag: 'Critical';
  values: void;
};

export type OwnerAlertConfig = {
  alert_level: AlertLevel;
  enabled: boolean;
};

export type WatcherInfo = {
  address: string;
  label: string;
  registered_at: bigint;
  registered_by: string;
};

export interface Client {
  register_watcher({ admin, watcher, label }: { admin: string; watcher: string; label?: string }): Promise<void>;
  get_watcher_info({ watcher }: { watcher: string }): Promise<WatcherInfo | undefined>;
  get_all_watchers(): Promise<string[]>;
  get_all_watchers_paginated({ offset, limit }: { offset: number; limit: number }): Promise<string[]>;
  get_admins(): Promise<string[]>;
  get_per_owner_alert_limit(): Promise<number | undefined>;
  get_owner_alert_config({ owner }: { owner: string }): Promise<OwnerAlertConfig | undefined>;
  set_per_owner_alert_limit({ limit }: { limit: number }): Promise<void>;
}

export class WatcherRegistry implements Client {
  readonly contract: Contract;

  constructor(contract: Contract) {
    this.contract = contract;
  }

  async get_per_owner_alert_limit(): Promise<number | undefined> {
    const result = await this.contract.call('get_per_owner_alert_limit');
    const value = result.result;
    return value === undefined || value === null ? undefined : Number(value);
  }

  async get_owner_alert_config({ owner }: { owner: string }): Promise<OwnerAlertConfig | undefined> {
    const result = await this.contract.call('get_owner_alert_config', { owner });
    const value = result.result;
    if (value === undefined || value === null) {
      return undefined;
    }
    return value as OwnerAlertConfig;
  }

  async set_per_owner_alert_limit({ limit }: { limit: number }): Promise<void> {
    await this.contract.call('set_per_owner_alert_limit', { limit });
  }

  async register_watcher({ admin, watcher, label }: { admin: string; watcher: string; label?: string }): Promise<void> {
    await this.contract.call('register_watcher', { admin, watcher, label });
  }

  async get_watcher_info({ watcher }: { watcher: string }): Promise<WatcherInfo | undefined> {
    const result = await this.contract.call('get_watcher_info', { watcher });
    const value = result.result;
    return value === undefined || value === null ? undefined : (value as WatcherInfo);
  }

  async get_all_watchers(): Promise<string[]> {
    const result = await this.contract.call('get_all_watchers');
    return result.result as string[];
  }

  async get_all_watchers_paginated({ offset, limit }: { offset: number; limit: number }): Promise<string[]> {
    const result = await this.contract.call('get_all_watchers_paginated', { offset, limit });
    return result.result as string[];
  }

  async get_admins(): Promise<string[]> {
    const result = await this.contract.call('get_admins');
    return result.result as string[];
  }
}
