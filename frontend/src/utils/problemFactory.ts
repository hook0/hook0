class ProblemFactory {
  public readonly status: number;
  public readonly type: string;
  public readonly title: string;
  public readonly detail: string;

  constructor(status: number, type: string, title: string, detail: string) {
    this.status = status;
    this.type = type;
    this.title = title;
    this.detail = detail;
  }
}

export default ProblemFactory;
