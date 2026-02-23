import { useState } from "react";
import { Channel } from "../types";

type Mode = "create" | "join";

interface ModalAddTopicProps {
  add_topic: (payload: string) => Promise<void>;
  setChannels: (channels: Channel[]) => void;
  setShowAddTopic: (show: boolean) => void;
  channels: Channel[];
  showAddTopic: boolean;
}

function ModalAddTopic(functions: ModalAddTopicProps) {
  const [mode, setMode] = useState<Mode>("create");
  const [topicName, setTopicName] = useState("");
  const [topicUuid, setTopicUuid] = useState("");

  const handleSubmit = async () => {
    // Validación: nombre siempre requerido
    if (!topicName.trim()) {
      alert("Please enter a topic name");
      return;
    }

    // Si es modo join, UUID también requerido
    if (mode === "join" && !topicUuid.trim()) {
      alert("Please enter a topic UUID to join");
      return;
    }

    // Determinar UUID según el modo
    const finalUuid = mode === "create" ? crypto.randomUUID() : topicUuid.trim();

    // Verificar que no exista ya un channel con ese UUID
    const exists = functions.channels.some((ch) => ch.uuid === finalUuid);
    if (exists) {
      alert("You're already in this topic!");
      return;
    }

    // Agregar el nuevo channel
    functions.setChannels([
      ...functions.channels,
      {
        name: topicName.trim(),
        unreadCount: 0,
        uuid: finalUuid,
      },
    ]);

    // Llamar al backend para suscribirse al topic
    try {
      await functions.add_topic(finalUuid);
      
      // Cerrar modal y resetear
      functions.setShowAddTopic(false);
      setTopicName("");
      setTopicUuid("");
      setMode("create");
    } catch (error) {
      console.error("Failed to add topic:", error);
      alert("Failed to add topic. Please try again.");
    }
  };

  const handleCancel = () => {
    functions.setShowAddTopic(false);
    setTopicName("");
    setTopicUuid("");
    setMode("create");
  };

  return (
    <div
      className={
        functions.showAddTopic
          ? "fixed inset-0 transition-opacity bg-black opacity-70 flex items-center justify-center z-50"
          : "hidden"
      }
      onClick={handleCancel} // Cerrar al hacer click fuera
    >
      <div
        className="bg-[#202020] p-6 rounded-lg w-96 shadow-2xl"
        onClick={(e) => e.stopPropagation()} // Evitar cerrar al hacer click dentro
      >
        {/* Header */}
        <h2 className="text-2xl mb-4 text-white font-semibold">
          {mode === "create" ? "Create New Topic" : "Join Existing Topic"}
        </h2>

        {/* Mode Selector */}
        <div className="flex gap-2 mb-4">
          <button
            onClick={() => setMode("create")}
            className={`flex-1 px-4 py-2 rounded transition ${
              mode === "create"
                ? "bg-neutral-800 text-white"
                : "bg-[#1a1a1a] text-gray-400 hover:bg-[#333]"
            }`}
          >
            🆕 Create
          </button>
          <button
            onClick={() => setMode("join")}
            className={`flex-1 px-4 py-2 rounded transition ${
              mode === "join"
                ? "bg-neutral-800 text-white"
                : "bg-[#1a1a1a] text-gray-400 hover:bg-[#333]"
            }`}
          >
            🔗 Join
          </button>
        </div>

        {/* Topic Name Input */}
        <div className="mb-3">
          <label className="block text-sm text-gray-400 mb-1">
            Topic Name
          </label>
          <input
            type="text"
            placeholder={mode === "create" ? "e.g., General Chat" : "Give it a name"}
            className="w-full p-3 rounded bg-[#303030] text-white focus:outline-none focus:ring-2 focus:ring-purple-600"
            value={topicName}
            onChange={(e) => setTopicName(e.target.value)}
            onKeyPress={(e) => e.key === "Enter" && handleSubmit()}
          />
        </div>

        {/* UUID Input (only in join mode) */}
        {mode === "join" && (
          <div className="mb-4">
            <label className="block text-sm text-gray-400 mb-1">
              Topic UUID
            </label>
            <input
              type="text"
              placeholder="Paste the UUID here"
              className="w-full p-3 rounded bg-[#303030] text-white font-mono text-sm focus:outline-none focus:ring-2 focus:ring-purple-600"
              value={topicUuid}
              onChange={(e) => setTopicUuid(e.target.value)}
              // onKeyPress={(e) => e.key === "Enter" && handleSubmit()}
            />
          </div>
        )}

        {/* Info Text */}
        <div className="mb-4 p-3 bg-neutral-800 rounded">
          <p className="text-xs text-purple-300">
            {mode === "create" ? (
              <>
                <strong>Create mode:</strong> A new unique UUID will be generated.
                Share it with others so they can join!
              </>
            ) : (
              <>
                <strong>Join mode:</strong> Enter the UUID someone shared with you
                to join their topic.
              </>
            )}
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex justify-end gap-2">
          <button
            onClick={handleCancel}
            className="px-4 py-2 rounded bg-[#1a1a1a] hover:bg-[#333] text-gray-100 transition"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            className="px-4 py-2 rounded bg-neutral-800 hover:bg-neutral-700 text-white font-semibold transition"
          >
            {mode === "create" ? "Create" : "Join"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default ModalAddTopic;