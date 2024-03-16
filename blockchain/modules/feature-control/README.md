# Feature Control Module

## Overview

This module provides a way of turning on/off features in the runtime that cannot be controlled with runtime configuration. It maintains a simple map of feature identifiers together with their status (enabled/disabled). It is supposed to be modified only by the specified origin, but read by any runtime code.
