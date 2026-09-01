import { Card, Empty } from "antd";
import { useTranslation } from "react-i18next";

export default function Settings() {
  const { t } = useTranslation();
  return (
    <Card title={t("nav.settings")}>
      <Empty description="Module coming soon" />
    </Card>
  );
}
