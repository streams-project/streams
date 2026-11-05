"""The two ways a stage stops."""


class StageFailed(Exception):
    """A stage could not do its work. The job ends with a non zero exit, so an alarm can fire."""


class FeatureOff(Exception):
    """A setting is missing, so this feature is off. The text says which one."""
