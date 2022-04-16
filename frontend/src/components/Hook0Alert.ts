export type AlertStatus = 'warning' | 'alert' | 'success';

export interface Alert {
  title: string,
  description: string,
  type: AlertStatus,
  visible: boolean,
}
