import React from 'react';

export default class ErrorBoundary extends React.Component {
    constructor(props) {
        super(props);
        this.state = { hasError: false, error: null, errorInfo: null };
    }

    static getDerivedStateFromError(error) {
        return { hasError: true };
    }

    componentDidCatch(error, errorInfo) {
        console.error('ErrorBoundary caught an error:', error, errorInfo);
        this.setState({
            error,
            errorInfo,
        });
    }

    render() {
        if (this.state.hasError) {
            return (
                <div style={{
                    minHeight: '100vh',
                    background: '#09111b',
                    color: '#e5edf7',
                    padding: '2rem',
                    fontFamily: 'monospace',
                }}>
                    <h1 style={{ color: '#f87171' }}>Something went wrong</h1>
                    <pre style={{
                        background: 'rgba(248, 113, 113, 0.1)',
                        padding: '1rem',
                        borderRadius: '8px',
                        overflow: 'auto',
                        maxHeight: '60vh'
                    }}>
                        {this.state.error?.toString()}
                    </pre>
                    {this.state.errorInfo?.componentStack && (
                        <details style={{ marginTop: '1rem' }}>
                            <summary style={{ cursor: 'pointer', color: '#60a5fa' }}>
                                Component Stack
                            </summary>
                            <pre style={{
                                background: 'rgba(96, 165, 250, 0.1)',
                                padding: '1rem',
                                borderRadius: '8px',
                                marginTop: '0.5rem',
                                fontSize: '12px'
                            }}>
                                {this.state.errorInfo.componentStack}
                            </pre>
                        </details>
                    )}
                    <button
                        onClick={() => window.location.reload()}
                        style={{
                            marginTop: '1rem',
                            padding: '0.5rem 1rem',
                            background: '#60a5fa',
                            border: 'none',
                            borderRadius: '6px',
                            color: '#09111b',
                            cursor: 'pointer',
                        }}
                    >
                        Reload Page
                    </button>
                </div>
            );
        }

        return this.props.children;
    }
}