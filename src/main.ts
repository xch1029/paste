import { createApp } from "vue";
import App from "./App.vue";
import PickerApp from "./PickerApp.vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./styles.css";

const currentWindow = getCurrentWebviewWindow();

createApp(currentWindow.label === "picker" ? PickerApp : App).mount("#app");
