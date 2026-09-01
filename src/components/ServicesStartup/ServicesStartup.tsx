import { Card, Empty } from "antd";
import { useTranslation } from "react-i18next";

export default function ServicesStartup() {
  const { t } = useTranslation();
  return (
    <Card title={t("nav.services")}>
      <Empty description="Module coming soon" />
    </Card>
  );
}
