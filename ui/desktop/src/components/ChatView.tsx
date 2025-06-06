import React, { useEffect, useRef, useState, useMemo } from 'react';
import { getApiUrl } from '../config';
import BottomMenu from './bottom_menu/BottomMenu';
import FlappyGoose from './FlappyGoose';
import GooseMessage from './GooseMessage';
import Input from './Input';
import { type View, ViewOptions } from '../App';
import LoadingGoose from './LoadingGoose';
import MoreMenuLayout from './more_menu/MoreMenuLayout';
import { Card } from './ui/card';
import { ScrollArea, ScrollAreaHandle } from './ui/scroll-area';
import UserMessage from './UserMessage';
import Splash from './Splash';
import { SearchView } from './conversation/SearchView';
import { createRecipe } from '../recipe';
import { AgentHeader } from './AgentHeader';
import LayingEggLoader from './LayingEggLoader';
import { fetchSessionDetails, generateSessionId } from '../sessions';
import 'react-toastify/dist/ReactToastify.css';
import { useMessageStream } from '../hooks/useMessageStream';
import { SessionSummaryModal } from './context_management/SessionSummaryModal';
import { Recipe } from '../recipe';
import {
  ChatContextManagerProvider,
  useChatContextManager,
} from './context_management/ContextManager';
import { ContextLengthExceededHandler } from './context_management/ContextLengthExceededHandler';
import {
  Message,
  createUserMessage,
  ToolCall,
  ToolCallResult,
  ToolRequestMessageContent,
  ToolResponseMessageContent,
  ToolConfirmationRequestMessageContent,
} from '../types/message';

export interface ChatType {
  id: string;
  title: string;
  // messages up to this index are presumed to be "history" from a resumed session, this is used to track older tool confirmation requests
  // anything before this index should not render any buttons, but anything after should
  messageHistoryIndex: number;
  messages: Message[];
}

// Helper function to determine if a message is a user message
const isUserMessage = (message: Message): boolean => {
  if (message.role === 'assistant') {
    return false;
  }
  if (message.content.every((c) => c.type === 'toolConfirmationRequest')) {
    return false;
  }
  return true;
};

export default function ChatView({
  chat,
  setChat,
  setView,
  setIsGoosehintsModalOpen,
}: {
  chat: ChatType;
  setChat: (chat: ChatType) => void;
  setView: (view: View, viewOptions?: ViewOptions) => void;
  setIsGoosehintsModalOpen: (isOpen: boolean) => void;
}) {
  return (
    <ChatContextManagerProvider>
      <ChatContent
        chat={chat}
        setChat={setChat}
        setView={setView}
        setIsGoosehintsModalOpen={setIsGoosehintsModalOpen}
      />
    </ChatContextManagerProvider>
  );
}

function ChatContent({
  chat,
  setChat,
  setView,
  setIsGoosehintsModalOpen,
}: {
  chat: ChatType;
  setChat: (chat: ChatType) => void;
  setView: (view: View, viewOptions?: ViewOptions) => void;
  setIsGoosehintsModalOpen: (isOpen: boolean) => void;
}) {
  // Disabled askAi calls to save costs
  // const [messageMetadata, setMessageMetadata] = useState<Record<string, string[]>>({});
  const [hasMessages, setHasMessages] = useState(false);
  const [lastInteractionTime, setLastInteractionTime] = useState<number>(Date.now());
  const [showGame, setShowGame] = useState(false);
  const [isGeneratingRecipe, setIsGeneratingRecipe] = useState(false);
  const [sessionTokenCount, setSessionTokenCount] = useState<number>(0);
  const [ancestorMessages, setAncestorMessages] = useState<Message[]>([]);
  const [droppedFiles, setDroppedFiles] = useState<string[]>([]);

  const scrollRef = useRef<ScrollAreaHandle>(null);

  const {
    summaryContent,
    summarizedThread,
    isSummaryModalOpen,
    resetMessagesWithSummary,
    closeSummaryModal,
    updateSummary,
    hasContextLengthExceededContent,
  } = useChatContextManager();

  useEffect(() => {
    // Log all messages when the component first mounts
    window.electron.logInfo(
      'Initial messages when resuming session: ' + JSON.stringify(chat.messages, null, 2)
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Empty dependency array means this runs once on mount;

  // Get recipeConfig directly from appConfig
  const recipeConfig = window.appConfig.get('recipeConfig') as Recipe | null;

  const {
    messages,
    append,
    stop,
    isLoading,
    error,
    setMessages,
    input: _input,
    setInput: _setInput,
    handleInputChange: _handleInputChange,
    handleSubmit: _submitMessage,
    updateMessageStreamBody,
  } = useMessageStream({
    api: getApiUrl('/reply'),
    initialMessages: chat.messages,
    body: { session_id: chat.id, session_working_dir: window.appConfig.get('GOOSE_WORKING_DIR') },
    onFinish: async (_message, _reason) => {
      window.electron.stopPowerSaveBlocker();

      setTimeout(() => {
        if (scrollRef.current?.scrollToBottom) {
          scrollRef.current.scrollToBottom();
        }
      }, 300);

      // Disabled askAi calls to save costs
      // const messageText = getTextContent(message);
      // const fetchResponses = await askAi(messageText);
      // setMessageMetadata((prev) => ({ ...prev, [message.id || '']: fetchResponses }));

      const timeSinceLastInteraction = Date.now() - lastInteractionTime;
      window.electron.logInfo('last interaction:' + lastInteractionTime);
      if (timeSinceLastInteraction > 60000) {
        // 60000ms = 1 minute
        window.electron.showNotification({
          title: 'Goose finished the task.',
          body: 'Click here to expand.',
        });
      }
    },
  });

  // for CLE events -- create a new session id for the next set of messages
  useEffect(() => {
    // If we're in a continuation session, update the chat ID
    if (summarizedThread.length > 0) {
      const newSessionId = generateSessionId();

      // Update the session ID in the chat object
      setChat({
        ...chat,
        id: newSessionId!,
        title: `Continued from ${chat.id}`,
        messageHistoryIndex: summarizedThread.length,
      });

      // Update the body used by useMessageStream to send future messages to the new session
      if (summarizedThread.length > 0 && updateMessageStreamBody) {
        updateMessageStreamBody({
          session_id: newSessionId,
          session_working_dir: window.appConfig.get('GOOSE_WORKING_DIR'),
        });
      }
    }

    // only update if summarizedThread length changes from 0 -> 1+
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    // eslint-disable-next-line react-hooks/exhaustive-deps
    summarizedThread.length > 0,
  ]);

  // Listen for make-agent-from-chat event
  useEffect(() => {
    const handleMakeAgent = async () => {
      window.electron.logInfo('Making recipe from chat...');
      setIsGeneratingRecipe(true);

      try {
        // Create recipe directly from chat messages
        const createRecipeRequest = {
          messages: messages,
          title: '',
          description: '',
        };

        const response = await createRecipe(createRecipeRequest);

        if (response.error) {
          throw new Error(`Failed to create recipe: ${response.error}`);
        }

        window.electron.logInfo('Created recipe:');
        window.electron.logInfo(JSON.stringify(response.recipe, null, 2));

        // First, verify the recipe data
        if (!response.recipe) {
          throw new Error('No recipe data received');
        }

        // Create a new window for the recipe editor
        console.log('Opening recipe editor with config:', response.recipe);
        window.electron.createChatWindow(
          undefined, // query
          undefined, // dir
          undefined, // version
          undefined, // resumeSessionId
          response.recipe, // recipe config
          'recipeEditor' // view type
        );

        window.electron.logInfo('Opening recipe editor window');
      } catch (error) {
        window.electron.logInfo('Failed to create recipe:');
        const errorMessage = error instanceof Error ? error.message : String(error);
        window.electron.logInfo(errorMessage);
      } finally {
        setIsGeneratingRecipe(false);
      }
    };

    window.addEventListener('make-agent-from-chat', handleMakeAgent);

    return () => {
      window.removeEventListener('make-agent-from-chat', handleMakeAgent);
    };
  }, [messages]);
  // do we need append here?
  // }, [append, chat.messages]);

  // Update chat messages when they change and save to sessionStorage
  useEffect(() => {
    setChat((prevChat) => {
      const updatedChat = { ...prevChat, messages };
      return updatedChat;
    });
  }, [messages, setChat]);

  useEffect(() => {
    if (messages.length > 0) {
      setHasMessages(true);
    }
  }, [messages]);

  // Handle submit
  const handleSubmit = (e: React.FormEvent) => {
    window.electron.startPowerSaveBlocker();
    const customEvent = e as unknown as CustomEvent;
    const content = customEvent.detail?.value || '';

    if (content.trim()) {
      setLastInteractionTime(Date.now());

      if (summarizedThread.length > 0) {
        // move current `messages` to `ancestorMessages` and `messages` to `summarizedThread`
        resetMessagesWithSummary(
          messages,
          setMessages,
          ancestorMessages,
          setAncestorMessages,
          summaryContent
        );

        // update the chat with new sessionId

        // now call the llm
        setTimeout(() => {
          append(createUserMessage(content));
          if (scrollRef.current?.scrollToBottom) {
            scrollRef.current.scrollToBottom();
          }
        }, 150);
      } else {
        // Normal flow (existing code)
        append(createUserMessage(content));
        if (scrollRef.current?.scrollToBottom) {
          scrollRef.current.scrollToBottom();
        }
      }
    }
  };

  if (error) {
    console.log('Error:', error);
  }

  const onStopGoose = () => {
    stop();
    setLastInteractionTime(Date.now());
    window.electron.stopPowerSaveBlocker();

    // Handle stopping the message stream
    const lastMessage = messages[messages.length - 1];

    // check if the last user message has any tool response(s)
    const isToolResponse = lastMessage.content.some(
      (content): content is ToolResponseMessageContent => content.type == 'toolResponse'
    );

    // isUserMessage also checks if the message is a toolConfirmationRequest
    // check if the last message is a real user's message
    if (lastMessage && isUserMessage(lastMessage) && !isToolResponse) {
      // Get the text content from the last message before removing it
      const textContent = lastMessage.content.find((c) => c.type === 'text')?.text || '';

      // Set the text back to the input field
      _setInput(textContent);

      // Remove the last user message if it's the most recent one
      if (messages.length > 1) {
        setMessages(messages.slice(0, -1));
      } else {
        setMessages([]);
      }
      // Interruption occured after a tool has completed, but no assistant reply
      // handle his if we want to popup a message too the user
      // } else if (lastMessage && isUserMessage(lastMessage) && isToolResponse) {
    } else if (!isUserMessage(lastMessage)) {
      // the last message was an assistant message
      // check if we have any tool requests or tool confirmation requests
      const toolRequests: [string, ToolCallResult<ToolCall>][] = lastMessage.content
        .filter(
          (content): content is ToolRequestMessageContent | ToolConfirmationRequestMessageContent =>
            content.type === 'toolRequest' || content.type === 'toolConfirmationRequest'
        )
        .map((content) => {
          if (content.type === 'toolRequest') {
            return [content.id, content.toolCall];
          } else {
            // extract tool call from confirmation
            const toolCall: ToolCallResult<ToolCall> = {
              status: 'success',
              value: {
                name: content.toolName,
                arguments: content.arguments,
              },
            };
            return [content.id, toolCall];
          }
        });

      if (toolRequests.length !== 0) {
        // This means we were interrupted during a tool request
        // Create tool responses for all interrupted tool requests

        let responseMessage: Message = {
          display: true,
          sendToLLM: true,
          role: 'user',
          created: Date.now(),
          content: [],
        };

        const notification = 'Interrupted by the user to make a correction';

        // generate a response saying it was interrupted for each tool request
        for (const [reqId, _] of toolRequests) {
          const toolResponse: ToolResponseMessageContent = {
            type: 'toolResponse',
            id: reqId,
            toolResult: {
              status: 'error',
              error: notification,
            },
          };

          responseMessage.content.push(toolResponse);
        }
        // Use an immutable update to add the response message to the messages array
        setMessages([...messages, responseMessage]);
      }
    }
  };

  // Filter out standalone tool response messages for rendering
  // They will be shown as part of the tool invocation in the assistant message
  const filteredMessages = [...ancestorMessages, ...messages].filter((message) => {
    // Only filter out when display is explicitly false
    if (message.display === false) return false;

    // Keep all assistant messages and user messages that aren't just tool responses
    if (message.role === 'assistant') return true;

    // For user messages, check if they're only tool responses
    if (message.role === 'user') {
      const hasOnlyToolResponses = message.content.every((c) => c.type === 'toolResponse');
      const hasTextContent = message.content.some((c) => c.type === 'text');
      const hasToolConfirmation = message.content.every(
        (c) => c.type === 'toolConfirmationRequest'
      );

      // Keep the message if it has text content or tool confirmation or is not just tool responses
      return hasTextContent || !hasOnlyToolResponses || hasToolConfirmation;
    }

    return true;
  });

  const commandHistory = useMemo(() => {
    return filteredMessages
      .reduce<string[]>((history, message) => {
        if (isUserMessage(message)) {
          const text = message.content.find((c) => c.type === 'text')?.text?.trim();
          if (text) {
            history.push(text);
          }
        }
        return history;
      }, [])
      .reverse();
  }, [filteredMessages]);

  // Fetch session metadata to get token count
  useEffect(() => {
    const fetchSessionTokens = async () => {
      try {
        const sessionDetails = await fetchSessionDetails(chat.id);
        setSessionTokenCount(sessionDetails.metadata.total_tokens);
      } catch (err) {
        console.error('Error fetching session token count:', err);
      }
    };
    if (chat.id) {
      fetchSessionTokens();
    }
  }, [chat.id, messages]);

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const paths: string[] = [];
      for (let i = 0; i < files.length; i++) {
        paths.push(window.electron.getPathForFile(files[i]));
      }
      setDroppedFiles(paths);
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  return (
    <div className="flex flex-col w-full h-screen items-center justify-center">
      {/* Loader when generating recipe */}
      {isGeneratingRecipe && <LayingEggLoader />}
      <div className="relative flex items-center h-[36px] w-full">
        <MoreMenuLayout setView={setView} setIsGoosehintsModalOpen={setIsGoosehintsModalOpen} />
      </div>

      <Card
        className="flex flex-col flex-1 rounded-none h-[calc(100vh-95px)] w-full bg-bgApp mt-0 border-none relative"
        onDrop={handleDrop}
        onDragOver={handleDragOver}
      >
        {recipeConfig?.title && messages.length > 0 && (
          <AgentHeader
            title={recipeConfig.title}
            profileInfo={
              recipeConfig.profile
                ? `${recipeConfig.profile} - ${recipeConfig.mcps || 12} MCPs`
                : undefined
            }
            onChangeProfile={() => {
              // Handle profile change
              console.log('Change profile clicked');
            }}
          />
        )}
        {messages.length === 0 ? (
          <Splash
            append={(text) => append(createUserMessage(text))}
            activities={Array.isArray(recipeConfig?.activities) ? recipeConfig.activities : null}
            title={recipeConfig?.title}
          />
        ) : (
          <ScrollArea ref={scrollRef} className="flex-1" autoScroll>
            <SearchView>
              {filteredMessages.map((message, index) => (
                <div
                  key={message.id || index}
                  className="mt-4 px-4"
                  data-testid="message-container"
                >
                  {isUserMessage(message) ? (
                    <UserMessage message={message} />
                  ) : (
                    <>
                      {/* Only render GooseMessage if it's not a CLE message (and we are not in alpha mode) */}
                      {process.env.NODE_ENV === 'development' &&
                      hasContextLengthExceededContent(message) ? (
                        <ContextLengthExceededHandler
                          messages={messages}
                          messageId={message.id ?? message.created.toString()}
                          chatId={chat.id}
                          workingDir={window.appConfig.get('GOOSE_WORKING_DIR') as string}
                        />
                      ) : (
                        <GooseMessage
                          messageHistoryIndex={chat?.messageHistoryIndex}
                          message={message}
                          messages={messages}
                          append={(text) => append(createUserMessage(text))}
                          appendMessage={(newMessage) => {
                            const updatedMessages = [...messages, newMessage];
                            setMessages(updatedMessages);
                          }}
                        />
                      )}
                    </>
                  )}
                </div>
              ))}
            </SearchView>
            {error && (
              <div className="flex flex-col items-center justify-center p-4">
                <div className="text-red-700 dark:text-red-300 bg-red-400/50 p-3 rounded-lg mb-2">
                  {error.message || 'Honk! Goose experienced an error while responding'}
                </div>
                <div
                  className="px-3 py-2 mt-2 text-center whitespace-nowrap cursor-pointer text-textStandard border border-borderSubtle hover:bg-bgSubtle rounded-full inline-block transition-all duration-150"
                  onClick={async () => {
                    // Find the last user message
                    const lastUserMessage = messages.reduceRight(
                      (found, m) => found || (m.role === 'user' ? m : null),
                      null as Message | null
                    );
                    if (lastUserMessage) {
                      append(lastUserMessage);
                    }
                  }}
                >
                  Retry Last Message
                </div>
              </div>
            )}
            <div className="block h-16" />
          </ScrollArea>
        )}

        <div className="relative">
          {isLoading && <LoadingGoose />}
          <Input
            handleSubmit={handleSubmit}
            isLoading={isLoading}
            onStop={onStopGoose}
            commandHistory={commandHistory}
            initialValue={_input}
            droppedFiles={droppedFiles}
          />
          <BottomMenu hasMessages={hasMessages} setView={setView} numTokens={sessionTokenCount} />
        </div>
      </Card>

      {showGame && <FlappyGoose onClose={() => setShowGame(false)} />}
      {process.env.NODE_ENV === 'development' && (
        <SessionSummaryModal
          isOpen={isSummaryModalOpen}
          onClose={closeSummaryModal}
          onSave={(editedContent) => {
            updateSummary(editedContent);
            closeSummaryModal();
          }}
          summaryContent={summaryContent}
        />
      )}
    </div>
  );
}
