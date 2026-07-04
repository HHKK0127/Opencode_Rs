import './ErrorToast.css'

interface ErrorToastProps {
  message: string
  onClose: () => void
}

export const ErrorToast = ({ message, onClose }: ErrorToastProps) => {
  return (
    <div data-component="error-toast">
      <span>{message}</span>
      <button onClick={onClose}>×</button>
    </div>
  )
}
