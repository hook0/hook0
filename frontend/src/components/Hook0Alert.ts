export type AlertStatus = 'warning' | 'alert' | 'success';

export type Alert = {
  title: string;
  description: string;
  type: AlertStatus;
  visible: boolean;
}
