import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { TypingIndicator } from './TypingIndicator'
import { Message } from '../types'
import './ChatContainer.css'

interface ChatContainerProps {
  messages: Message[]
  messagesEndRef: React.RefObject<HTMLDivElement | null>
  isLoading?: boolean
}

const CodeBlock = ({ language, value }: { language: string; value: string }) => (
  <div data-component="code-block">
    <div data-slot="code-header">{language}</div>
    <SyntaxHighlighter language={language} style={vscDarkPlus}>
      {value}
    </SyntaxHighlighter>
  </div>
)

export const ChatContainer = ({ messages, messagesEndRef, isLoading }: ChatContainerProps) => {
  return (
    <div data-component="session-turn">
      <div data-slot="session-turn-content">
        {messages.length === 0 && (
          <div className="empty-state">
            <p>Start a conversation with OpenCode</p>
          </div>
        )}
        {messages.map((msg) => (
          <div
            key={msg.id}
            data-component={msg.role === 'user' ? 'user-message' : 'assistant-message'}
          >
            {msg.role === 'user' ? (
              <div data-slot="user-message-bubble">{msg.content}</div>
            ) : (
              <div data-slot="message-content">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  components={{
                    code({ inline, className, children, ...props }: any) {
                      const match = /language-(\w+)/.exec(className || '')
                      return !inline && match ? (
                        <CodeBlock language={match[1]} value={String(children).replace(/\n$/, '')} />
                      ) : (
                        <code className="inline-code" {...props}>
                          {children}
                        </code>
                      )
                    },
                  }}
                >
                  {msg.content}
                </ReactMarkdown>
              </div>
            )}
          </div>
        ))}
        {isLoading && (
          <div data-component="assistant-message">
            <TypingIndicator />
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
    </div>
  )
}
