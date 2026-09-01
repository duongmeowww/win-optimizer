import { Card, Empty } from "antd";
import { useTranslation } from "react-i18next";

export default function MemoryCleaner() {
  const { t } = useTranslation();
  return (
    <Card title={t("nav.memory")}>
      <Empty description="Module coming soon" />
    </Card>
  );
}
