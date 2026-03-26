
interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  text?: string;
}

export default function LoadingSpinner({ size = 'md', text }: LoadingSpinnerProps) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-8 w-8',
    lg: 'h-12 w-12',
  };

  return (
    <div className="flex flex-col items-center justify-center py-12">
      <div className={`animate-spin rounded-full border-b-2 border-blue-600 ${sizeClasses[size]}`}></div>
      {text && (
        <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">{text}</p>
      )}
    </div>
  );
}
