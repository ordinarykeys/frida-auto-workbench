import i18n from "i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import { initReactI18next } from "react-i18next";

const resources = {
  zh: {
    translation: {
      appTitle: "Frida 自动分析工作台",
      appSubtitle: "面向已 Root 安卓设备的 Frida 控制、APK 元数据分析与 AI 自动脚本生成",
      rootedDevice: "已 Root 设备",
      automation: "自动分析",
    },
  },
};

i18n.use(LanguageDetector).use(initReactI18next).init({
  resources,
  lng: "zh",
  fallbackLng: "zh",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
