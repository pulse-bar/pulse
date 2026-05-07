import React from 'react';

interface State {
  error: Error | null;
}

interface Props {
  label: string;
  children: React.ReactNode;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error(`ErrorBoundary[${this.props.label}]`, error, info);
  }

  reset = () => {
    this.setState({ error: null });
  };

  render() {
    if (this.state.error) {
      return (
        <div
          style={{
            background: 'var(--red-d)',
            border: '1px solid rgba(220, 70, 70, 0.3)',
            borderRadius: 8,
            padding: 16,
            fontFamily: 'var(--mono)',
            fontSize: 11,
            color: 'var(--red)',
            display: 'flex',
            flexDirection: 'column',
            gap: 10,
          }}
        >
          <div style={{ fontWeight: 700 }}>{this.props.label} failed to render.</div>
          <div style={{ color: 'var(--txt2)', fontSize: 10, lineHeight: 1.5 }}>
            {this.state.error.message}
          </div>
          <button
            className="btn"
            onClick={this.reset}
            style={{ alignSelf: 'flex-start' }}
          >
            Retry
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
