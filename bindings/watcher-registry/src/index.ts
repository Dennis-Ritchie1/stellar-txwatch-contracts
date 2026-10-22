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

export interface Client {
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
}
