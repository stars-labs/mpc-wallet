/**
 * Logger utility for MPC Wallet
 * Provides environment-aware logging with configurable levels
 */

export enum LogLevel {
  ERROR = 0,
  WARN = 1,
  INFO = 2,
  DEBUG = 3,
}

export interface LogContext {
  component?: string;
  method?: string;
  [key: string]: any;
}

class Logger {
  private static instance: Logger;
  private logLevel: LogLevel;
  private isDevelopment: boolean;

  private constructor() {
    // Check if we're in development mode
    this.isDevelopment = process.env.NODE_ENV === 'development' || 
                        (typeof chrome !== 'undefined' && chrome.runtime && 
                         chrome.runtime.getManifest().version?.includes('dev'));
    
    // Set default log level based on environment
    this.logLevel = this.isDevelopment ? LogLevel.DEBUG : LogLevel.WARN;
    
    // Allow override via environment variable
    const envLogLevel = process.env.LOG_LEVEL;
    if (envLogLevel && LogLevel[envLogLevel as keyof typeof LogLevel] !== undefined) {
      this.logLevel = LogLevel[envLogLevel as keyof typeof LogLevel];
    }
  }

  static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
    }
    return Logger.instance;
  }

  setLogLevel(level: LogLevel): void {
    this.logLevel = level;
  }

  private shouldLog(level: LogLevel): boolean {
    return level <= this.logLevel;
  }

  private formatMessage(level: string, message: string, context?: LogContext): string {
    const timestamp = new Date().toISOString();
    const component = context?.component ? `[${context.component}]` : '';
    const method = context?.method ? `.${context.method}` : '';
    return `${timestamp} ${level} ${component}${method} ${message}`;
  }

  private log(level: LogLevel, levelStr: string, message: string, context?: LogContext, ...args: any[]): void {
    if (!this.shouldLog(level)) return;

    const formattedMessage = this.formatMessage(levelStr, message, context);
    
    switch (level) {
      case LogLevel.ERROR:
        console.error(formattedMessage, ...args);
        break;
      case LogLevel.WARN:
        console.warn(formattedMessage, ...args);
        break;
      case LogLevel.INFO:
        console.info(formattedMessage, ...args);
        break;
      case LogLevel.DEBUG:
        if (this.isDevelopment) {
          console.log(formattedMessage, ...args);
        }
        break;
    }

    // In production, send errors to monitoring service
    if (!this.isDevelopment && level === LogLevel.ERROR) {
      this.sendToMonitoring(formattedMessage, context, args);
    }
  }

  private sendToMonitoring(message: string, context?: LogContext, args?: any[]): void {
    // TODO: Implement error reporting to monitoring service
    // This could send to Sentry, LogRocket, or custom analytics
  }

  error(message: string, context?: LogContext, ...args: any[]): void {
    this.log(LogLevel.ERROR, 'ERROR', message, context, ...args);
  }

  warn(message: string, context?: LogContext, ...args: any[]): void {
    this.log(LogLevel.WARN, 'WARN', message, context, ...args);
  }

  info(message: string, context?: LogContext, ...args: any[]): void {
    this.log(LogLevel.INFO, 'INFO', message, context, ...args);
  }

  debug(message: string, context?: LogContext, ...args: any[]): void {
    this.log(LogLevel.DEBUG, 'DEBUG', message, context, ...args);
  }

  // Convenience method for logging security events (always logged)
  security(event: string, details: any): void {
    const message = `SECURITY EVENT: ${event}`;
    console.warn(this.formatMessage('SECURITY', message), details);
    
    // Always send security events to monitoring in production
    if (!this.isDevelopment) {
      this.sendToMonitoring(message, { type: 'security', event }, details);
    }
  }

  // Convenience method for logging audit events (always logged)
  audit(action: string, details: any): void {
    const message = `AUDIT: ${action}`;
    console.info(this.formatMessage('AUDIT', message), details);
  }
}

// Export singleton instance
export const logger = Logger.getInstance();

// Export for testing
export { Logger };