import { Show } from "solid-js";
import RecordingToolbar from "./components/RecordingToolbar";
import ReviewScreen from "./components/ReviewScreen";
import "./App.css";

function App() {
  const isReview = window.location.pathname === "/review";

  return (
    <Show when={!isReview} fallback={<ReviewScreen />}>
      <RecordingToolbar />
    </Show>
  );
}

export default App;
